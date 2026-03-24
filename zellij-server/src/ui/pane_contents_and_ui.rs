use crate::output::{CharacterChunk, Output};
use crate::panes::{AnsiCode, PaneId, RcCharacterStyles, TerminalCharacter};
use crate::tab::Pane;
use crate::ui::boundaries::{boundary_type, Boundaries};
use crate::ui::pane_boundaries_frame::FrameParams;
use crate::ClientId;
use std::collections::{HashMap, HashSet};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use zellij_utils::data::{
    client_id_to_colors, HeaderAlignment, InputMode, PaletteColor, StackedPaneHeaderAction,
    StackedPaneHeaderItemStyle, StackedPaneHeaderSpec, Style,
};
use zellij_utils::errors::prelude::*;
use zellij_utils::pane_size::PaneGeom;
#[derive(Clone, Debug)]
pub struct StackedPaneTab {
    pub pane_id: PaneId,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct StackedPaneHeader {
    pub full_stack_geom: PaneGeom,
    pub stack_id: usize,
    pub expanded_pane_id: PaneId,
    pub tabs: Vec<StackedPaneTab>,
}

impl StackedPaneHeader {
    pub fn expanded_pane_id(&self) -> Option<PaneId> {
        Some(self.expanded_pane_id)
    }

    fn visible_tab_count(&self) -> usize {
        let inner_width = self.full_stack_geom.cols.as_usize().saturating_sub(2);
        let mut visible_tab_count = self.tabs.len();

        while visible_tab_count > 0 {
            let minimum_width_for_visible_tabs = visible_tab_count.saturating_mul(2) + 1;
            if minimum_width_for_visible_tabs <= inner_width {
                break;
            }
            visible_tab_count -= 1;
        }
        visible_tab_count
    }

    fn tab_label_widths(&self, visible_tab_count: usize) -> Vec<usize> {
        if visible_tab_count == 0 {
            return vec![];
        }

        let inner_width = self.full_stack_geom.cols.as_usize().saturating_sub(2);
        let available_label_width = inner_width.saturating_sub(visible_tab_count + 1);
        let ideal_widths: Vec<usize> = self
            .tabs
            .iter()
            .take(visible_tab_count)
            .map(|tab| tab.title.width() + 2)
            .collect();
        let mut label_widths = vec![1; visible_tab_count];
        let mut remaining_width = available_label_width.saturating_sub(visible_tab_count);

        while remaining_width > 0 {
            let mut widened_a_tab = false;
            for (label_width, ideal_width) in label_widths.iter_mut().zip(ideal_widths.iter()) {
                if *label_width < *ideal_width {
                    *label_width += 1;
                    remaining_width -= 1;
                    widened_a_tab = true;
                }
                if remaining_width == 0 {
                    break;
                }
            }
            if !widened_a_tab {
                break;
            }
        }
        label_widths
    }

    pub fn tab_boundaries(&self) -> Vec<(PaneId, usize, usize)> {
        let visible_tab_count = self.visible_tab_count();
        let tab_label_widths = self.tab_label_widths(visible_tab_count);
        let mut start = 0;

        self.tabs
            .iter()
            .take(visible_tab_count)
            .zip(tab_label_widths)
            .enumerate()
            .map(|(i, (tab, label_width))| {
                let end = start + 1 + label_width + usize::from(i + 1 == visible_tab_count);
                let boundary = (tab.pane_id, start, end);
                start = end;
                boundary
            })
            .collect()
    }

    pub fn pane_id_at(&self, column: usize) -> Option<PaneId> {
        self.tab_boundaries()
            .into_iter()
            .find(|(_pane_id, start, end)| column >= *start && column < *end)
            .map(|(pane_id, _start, _end)| pane_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedStackedPaneHeaderSegment {
    pub start: usize,
    pub end: usize,
    pub pane_id: Option<PaneId>,
    pub action: Option<StackedPaneHeaderAction>,
    pub style: StackedPaneHeaderItemStyle,
    pub segment: String,
}

fn truncate_text_to_width(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if text.width() <= width {
        return text.to_owned();
    }
    if width == 1 {
        return "…".to_owned();
    }

    let mut truncated = String::new();
    let mut current_width = 0;
    for character in text.chars() {
        let character_width = character.width().unwrap_or(0);
        if current_width + character_width + 1 > width {
            break;
        }
        truncated.push(character);
        current_width += character_width;
    }
    truncated.push('…');
    truncated
}

fn stacked_pane_header_item_ideal_width(
    text: &str,
    _style: StackedPaneHeaderItemStyle,
) -> usize {
    text.width().saturating_add(2)
}

fn stacked_pane_header_item_label(
    text: &str,
    width: usize,
    style: StackedPaneHeaderItemStyle,
) -> String {
    if width == 0 {
        return String::new();
    }

    match style {
        StackedPaneHeaderItemStyle::Selected => {
            if width == 1 {
                return "…".to_owned();
            }
            let truncated_title = truncate_text_to_width(text, width.saturating_sub(2));
            let wrapped_title = format!("[{}]", truncated_title);
            if wrapped_title.width() <= width {
                wrapped_title
            } else {
                truncate_text_to_width(&wrapped_title, width)
            }
        },
        _ => {
            let padded_title = format!(" {} ", text);
            if padded_title.width() <= width {
                padded_title
            } else {
                truncate_text_to_width(text, width)
            }
        },
    }
}

pub fn stacked_pane_header_segments(
    stacked_pane_header: &StackedPaneHeader,
    spec: &StackedPaneHeaderSpec,
 ) -> Vec<RenderedStackedPaneHeaderSegment> {
    let inner_width = stacked_pane_header.full_stack_geom.cols.as_usize().saturating_sub(2);
    if inner_width == 0 || spec.items.is_empty() {
        return vec![];
    }

    let mut visible_item_count = spec.items.len();
    while visible_item_count > 0 {
        let minimum_width = visible_item_count.saturating_mul(2).saturating_add(1);
        if minimum_width <= inner_width {
            break;
        }
        visible_item_count -= 1;
    }
    if visible_item_count == 0 {
        return vec![];
    }

    let available_label_width = inner_width.saturating_sub(visible_item_count + 1);
    let ideal_widths: Vec<usize> = spec
        .items
        .iter()
        .take(visible_item_count)
        .map(|item| stacked_pane_header_item_ideal_width(&item.text, item.style))
        .collect();
    let mut label_widths = vec![1; visible_item_count];
    let mut remaining_width = available_label_width.saturating_sub(visible_item_count);
    while remaining_width > 0 {
        let mut widened_an_item = false;
        for (label_width, ideal_width) in label_widths.iter_mut().zip(ideal_widths.iter()) {
            if *label_width < *ideal_width {
                *label_width += 1;
                remaining_width -= 1;
                widened_an_item = true;
            }
            if remaining_width == 0 {
                break;
            }
        }
        if !widened_an_item {
            break;
        }
    }

    let group_width = visible_item_count + label_widths.iter().sum::<usize>() + 1;
    let mut current_start = match spec.alignment {
        HeaderAlignment::Left => 0,
        HeaderAlignment::Center => inner_width.saturating_sub(group_width) / 2,
        HeaderAlignment::Right => inner_width.saturating_sub(group_width),
    };

    spec.items
        .iter()
        .take(visible_item_count)
        .zip(label_widths.into_iter())
        .enumerate()
        .map(|(i, (item, label_width))| {
            let include_trailing_separator = i + 1 == visible_item_count;
            let label = stacked_pane_header_item_label(&item.text, label_width, item.style);
            let right_padding = label_width.saturating_sub(label.width());
            let trailing_separator = if include_trailing_separator {
                boundary_type::VERTICAL
            } else {
                ""
            };
            let segment = format!(
                "{}{}{}{}",
                boundary_type::VERTICAL,
                label,
                boundary_type::HORIZONTAL.repeat(right_padding),
                trailing_separator,
            );
            let start = current_start;
            let end = start + segment.width();
            current_start = end;
            RenderedStackedPaneHeaderSegment {
                start,
                end,
                pane_id: item.pane_id.map(Into::into),
                action: item.action.clone(),
                style: item.style,
                segment,
            }
        })
        .collect()
}


pub struct PaneContentsAndUi<'a> {
    pane: &'a mut Box<dyn Pane>,
    output: &'a mut Output,
    style: Style,
    focused_clients: Vec<ClientId>,
    multiple_users_exist_in_session: bool,
    z_index: Option<usize>,
    pane_is_stacked_under: bool,
    pane_is_stacked_over: bool,
    should_draw_pane_frames: bool,
    mouse_is_hovering_over_pane_for_clients: HashSet<ClientId>,
    current_pane_group: HashMap<ClientId, Vec<PaneId>>,
    show_help_text: bool,
}

fn styled_characters(characters: &str, color: Option<PaletteColor>) -> Vec<TerminalCharacter> {
    let mut colored_string = Vec::new();
    for character in characters.chars() {
        let mut styles = RcCharacterStyles::reset();
        styles.update(|styles| {
            styles.bold = Some(AnsiCode::On);
            if let Some(palette_color) = color {
                styles.foreground = Some(AnsiCode::from(palette_color));
            }
        });
        colored_string.push(TerminalCharacter::new_styled(character, styles));
    }
    colored_string
}

impl<'a> PaneContentsAndUi<'a> {
    pub fn new(
        pane: &'a mut Box<dyn Pane>,
        output: &'a mut Output,
        style: Style,
        active_panes: &HashMap<ClientId, PaneId>,
        multiple_users_exist_in_session: bool,
        z_index: Option<usize>,
        pane_is_stacked_under: bool,
        pane_is_stacked_over: bool,
        should_draw_pane_frames: bool,
        mouse_hover_pane_id: &HashMap<ClientId, PaneId>,
        current_pane_group: HashMap<ClientId, Vec<PaneId>>,
        show_help_text: bool,
    ) -> Self {
        let mut focused_clients: Vec<ClientId> = active_panes
            .iter()
            .filter(|(_c_id, p_id)| **p_id == pane.pid())
            .map(|(c_id, _p_id)| *c_id)
            .collect();
        focused_clients.sort_unstable();
        let mouse_is_hovering_over_pane_for_clients = mouse_hover_pane_id
            .iter()
            .filter_map(|(client_id, pane_id)| {
                if pane_id == &pane.pid() {
                    Some(*client_id)
                } else {
                    None
                }
            })
            .collect();
        PaneContentsAndUi {
            pane,
            output,
            style,
            focused_clients,
            multiple_users_exist_in_session,
            z_index,
            pane_is_stacked_under,
            pane_is_stacked_over,
            should_draw_pane_frames,
            mouse_is_hovering_over_pane_for_clients,
            current_pane_group,
            show_help_text,
        }
    }
    pub fn render_pane_contents_to_multiple_clients(
        &mut self,
        clients: impl Iterator<Item = ClientId>,
    ) -> Result<()> {
        let err_context = "failed to render pane contents to multiple clients";

        // here we drop the fake cursors so that their lines will be updated
        // and we can clear them from the UI below
        drop(self.pane.drain_fake_cursors());

        if let Some((character_chunks, raw_vte_output, sixel_image_chunks)) =
            self.pane.render(None).context(err_context)?
        {
            let clients: Vec<ClientId> = clients.collect();
            self.output
                .add_character_chunks_to_multiple_clients(
                    character_chunks,
                    clients.iter().copied(),
                    self.z_index,
                )
                .context(err_context)?;
            self.output.add_sixel_image_chunks_to_multiple_clients(
                sixel_image_chunks,
                clients.iter().copied(),
                self.z_index,
            );
            if let Some(raw_vte_output) = raw_vte_output {
                if !raw_vte_output.is_empty() {
                    self.output.add_post_vte_instruction_to_multiple_clients(
                        clients.iter().copied(),
                        &format!(
                            "\u{1b}[{};{}H\u{1b}[m{}",
                            self.pane.y() + 1,
                            self.pane.x() + 1,
                            raw_vte_output
                        ),
                    );
                }
            }
        }
        Ok(())
    }
    pub fn render_pane_contents_for_client(&mut self, client_id: ClientId) -> Result<()> {
        let err_context = || format!("failed to render pane contents for client {client_id}");

        if let Some((character_chunks, raw_vte_output, sixel_image_chunks)) = self
            .pane
            .render(Some(client_id))
            .with_context(err_context)?
        {
            self.output
                .add_character_chunks_to_client(client_id, character_chunks, self.z_index)
                .with_context(err_context)?;
            self.output.add_sixel_image_chunks_to_client(
                client_id,
                sixel_image_chunks,
                self.z_index,
            );
            if let Some(raw_vte_output) = raw_vte_output {
                self.output.add_post_vte_instruction_to_client(
                    client_id,
                    &format!(
                        "\u{1b}[{};{}H\u{1b}[m{}",
                        self.pane.y() + 1,
                        self.pane.x() + 1,
                        raw_vte_output
                    ),
                );
            }
        }
        Ok(())
    }
    pub fn render_fake_cursor_if_needed(&mut self, client_id: ClientId) -> Result<()> {
        let pane_focused_for_client_id = self.focused_clients.contains(&client_id);
        let pane_focused_for_different_client = self
            .focused_clients
            .iter()
            .filter(|&&c_id| c_id != client_id)
            .count()
            > 0;
        if pane_focused_for_different_client && !pane_focused_for_client_id {
            let fake_cursor_client_id = self
                .focused_clients
                .iter()
                .find(|&&c_id| c_id != client_id)
                .with_context(|| {
                    format!("failed to render fake cursor if needed for client {client_id}")
                })?;
            if let Some(colors) = client_id_to_colors(
                *fake_cursor_client_id,
                self.style.colors.multiplayer_user_colors,
            ) {
                let cursor_is_visible = self
                    .pane
                    .cursor_coordinates(Some(*fake_cursor_client_id))
                    .map(|(x, y)| {
                        self.output.cursor_is_visible(
                            self.pane.x() + x,
                            self.pane.y() + y,
                            self.z_index,
                        )
                    })
                    .unwrap_or(false);
                if cursor_is_visible {
                    if let Some(vte_output) = self.pane.render_fake_cursor(colors.0, colors.1) {
                        self.output.add_post_vte_instruction_to_client(
                            client_id,
                            &format!(
                                "\u{1b}[{};{}H\u{1b}[m{}",
                                self.pane.y() + 1,
                                self.pane.x() + 1,
                                vte_output
                            ),
                        );
                    }
                }
            }
        }
        Ok(())
    }
    pub fn render_terminal_title_if_needed(
        &mut self,
        client_id: ClientId,
        client_mode: InputMode,
        previous_title: &mut Option<String>,
    ) {
        if !self.focused_clients.contains(&client_id) {
            return;
        }
        let vte_output = self.pane.render_terminal_title(client_mode);
        if let Some(previous_title) = previous_title {
            if *previous_title == vte_output {
                return;
            }
        }
        *previous_title = Some(vte_output.clone());
        self.output
            .add_post_vte_instruction_to_client(client_id, &vte_output);
    }
    pub fn render_pane_frame(
        &mut self,
        client_id: ClientId,
        client_mode: InputMode,
        session_is_mirrored: bool,
        pane_is_floating: bool,
        pane_is_selectable: bool,
    ) -> Result<()> {
        let err_context = || format!("failed to render pane frame for client {client_id}");

        let pane_focused_for_client_id = self.focused_clients.contains(&client_id);
        let other_focused_clients: Vec<ClientId> = self
            .focused_clients
            .iter()
            .filter(|&&c_id| c_id != client_id)
            .copied()
            .collect();
        let pane_focused_for_differet_client = !other_focused_clients.is_empty();

        let frame_color = self.frame_color(client_id, client_mode, session_is_mirrored);
        let highlight_tooltip = self.pane.cached_hover_tooltip();
        let focused_client = if pane_focused_for_client_id {
            Some(client_id)
        } else if pane_focused_for_differet_client {
            Some(*other_focused_clients.first().with_context(err_context)?)
        } else {
            None
        };
        let frame_params = if session_is_mirrored {
            FrameParams {
                focused_client,
                is_main_client: pane_focused_for_client_id,
                other_focused_clients: vec![],
                style: self.style,
                color: frame_color.map(|c| c.0),
                other_cursors_exist_in_session: false,
                pane_is_stacked_over: self.pane_is_stacked_over,
                pane_is_stacked_under: self.pane_is_stacked_under,
                should_draw_pane_frames: self.should_draw_pane_frames,
                pane_is_floating,
                content_offset: self.pane.get_content_offset(),
                mouse_is_hovering_over_pane: self
                    .mouse_is_hovering_over_pane_for_clients
                    .contains(&client_id),
                pane_is_selectable,
                show_help_text: self.show_help_text,
                highlight_tooltip: highlight_tooltip.clone(),
            }
        } else {
            FrameParams {
                focused_client,
                is_main_client: pane_focused_for_client_id,
                other_focused_clients,
                style: self.style,
                color: frame_color.map(|c| c.0),
                other_cursors_exist_in_session: self.multiple_users_exist_in_session,
                pane_is_stacked_over: self.pane_is_stacked_over,
                pane_is_stacked_under: self.pane_is_stacked_under,
                should_draw_pane_frames: self.should_draw_pane_frames,
                pane_is_floating,
                content_offset: self.pane.get_content_offset(),
                mouse_is_hovering_over_pane: self
                    .mouse_is_hovering_over_pane_for_clients
                    .contains(&client_id),
                pane_is_selectable,
                show_help_text: self.show_help_text,
                highlight_tooltip,
            }
        };

        if let Some((frame_terminal_characters, vte_output)) = self
            .pane
            .render_frame(client_id, frame_params, client_mode)
            .with_context(err_context)?
        {
            self.output
                .add_character_chunks_to_client(client_id, frame_terminal_characters, self.z_index)
                .with_context(err_context)?;
            if let Some(vte_output) = vte_output {
                self.output
                    .add_post_vte_instruction_to_client(client_id, &vte_output);
            }
        }

        Ok(())
    }
    pub fn render_stacked_pane_header(
        &mut self,
        client_id: ClientId,
        client_mode: InputMode,
        session_is_mirrored: bool,
        stacked_pane_header: &StackedPaneHeader,
        selected_pane_id: Option<PaneId>,
        stacked_pane_header_spec: Option<&StackedPaneHeaderSpec>,
    ) -> Result<()> {
        let color = self
            .frame_color(client_id, client_mode, session_is_mirrored)
            .map(|(color, _precedence)| color);
        let header_line = match stacked_pane_header_spec {
            Some(stacked_pane_header_spec) => self.stacked_pane_header_line_from_spec(
                stacked_pane_header,
                stacked_pane_header_spec,
                color,
            ),
            None => self.stacked_pane_header_line(stacked_pane_header, color, selected_pane_id),
        };
        self.output
            .add_character_chunks_to_client(
                client_id,
                vec![CharacterChunk::new(
                    header_line,
                    stacked_pane_header.full_stack_geom.x,
                    stacked_pane_header.full_stack_geom.y,
                )],
                self.z_index,
            )
            .with_context(|| format!("failed to render stacked pane header for client {client_id}"))
    }

    fn stacked_pane_header_line(
        &self,
        stacked_pane_header: &StackedPaneHeader,
        color: Option<PaletteColor>,
        selected_pane_id: Option<PaneId>,
    ) -> Vec<TerminalCharacter> {
        let total_width = stacked_pane_header.full_stack_geom.cols.as_usize();
        let inner_width = total_width.saturating_sub(2);
        let left_boundary = if self.should_draw_pane_frames {
            if self.style.rounded_corners {
                boundary_type::TOP_LEFT_ROUND
            } else {
                boundary_type::TOP_LEFT
            }
        } else {
            boundary_type::HORIZONTAL
        };
        let right_boundary = if self.should_draw_pane_frames {
            if self.style.rounded_corners {
                boundary_type::TOP_RIGHT_ROUND
            } else {
                boundary_type::TOP_RIGHT
            }
        } else {
            boundary_type::HORIZONTAL
        };
        let tab_boundaries = stacked_pane_header.tab_boundaries();
        let mut line = styled_characters(left_boundary, color);
        let mut rendered_columns = 0;

        for (i, (tab, (_pane_id, start, end))) in stacked_pane_header
            .tabs
            .iter()
            .zip(tab_boundaries.iter())
            .enumerate()
        {
            if *start > rendered_columns {
                let filler = boundary_type::HORIZONTAL.repeat(*start - rendered_columns);
                line.append(&mut styled_characters(&filler, color));
            }

            let include_trailing_separator = i + 1 == tab_boundaries.len();
            let label_width = end
                .saturating_sub(*start)
                .saturating_sub(1 + usize::from(include_trailing_separator));
            let segment = self.stacked_pane_header_segment(
                &tab.title,
                label_width,
                selected_pane_id == Some(tab.pane_id),
                include_trailing_separator,
            );
            line.append(&mut styled_characters(&segment, color));
            rendered_columns = *end;
        }

        if inner_width > rendered_columns {
            let filler = boundary_type::HORIZONTAL.repeat(inner_width - rendered_columns);
            line.append(&mut styled_characters(&filler, color));
        }
        line.append(&mut styled_characters(right_boundary, color));
        line
    }

    fn stacked_pane_header_item_color(
        &self,
        style: StackedPaneHeaderItemStyle,
        default_color: Option<PaletteColor>,
    ) -> Option<PaletteColor> {
        match style {
            StackedPaneHeaderItemStyle::Default => default_color,
            StackedPaneHeaderItemStyle::Selected => Some(self.style.colors.frame_selected.base),
            StackedPaneHeaderItemStyle::Warning => Some(self.style.colors.exit_code_error.base),
            StackedPaneHeaderItemStyle::Success => Some(self.style.colors.exit_code_success.base),
            StackedPaneHeaderItemStyle::Muted => self
                .style
                .colors
                .frame_unselected
                .map(|style| style.base)
                .or(Some(self.style.colors.frame_highlight.base)),
        }
    }

    fn stacked_pane_header_line_from_spec(
        &self,
        stacked_pane_header: &StackedPaneHeader,
        spec: &StackedPaneHeaderSpec,
        default_color: Option<PaletteColor>,
    ) -> Vec<TerminalCharacter> {
        let total_width = stacked_pane_header.full_stack_geom.cols.as_usize();
        let inner_width = total_width.saturating_sub(2);
        let left_boundary = if self.should_draw_pane_frames {
            if self.style.rounded_corners {
                boundary_type::TOP_LEFT_ROUND
            } else {
                boundary_type::TOP_LEFT
            }
        } else {
            boundary_type::HORIZONTAL
        };
        let right_boundary = if self.should_draw_pane_frames {
            if self.style.rounded_corners {
                boundary_type::TOP_RIGHT_ROUND
            } else {
                boundary_type::TOP_RIGHT
            }
        } else {
            boundary_type::HORIZONTAL
        };
        let mut line = styled_characters(left_boundary, default_color);
        let mut rendered_columns = 0;
        for segment in stacked_pane_header_segments(stacked_pane_header, spec) {
            if segment.start > rendered_columns {
                let filler = boundary_type::HORIZONTAL.repeat(segment.start - rendered_columns);
                line.append(&mut styled_characters(&filler, default_color));
            }
            line.append(&mut styled_characters(
                &segment.segment,
                self.stacked_pane_header_item_color(segment.style, default_color),
            ));
            rendered_columns = segment.end;
        }
        if inner_width > rendered_columns {
            let filler = boundary_type::HORIZONTAL.repeat(inner_width - rendered_columns);
            line.append(&mut styled_characters(&filler, default_color));
        }
        line.append(&mut styled_characters(right_boundary, default_color));
        line
    }


    fn stacked_pane_header_segment(
        &self,
        title: &str,
        label_width: usize,
        is_selected: bool,
        include_trailing_separator: bool,
    ) -> String {
        let label = self.stacked_pane_label(title, label_width, is_selected);
        let right_padding = label_width.saturating_sub(label.width());
        let trailing_separator = if include_trailing_separator {
            boundary_type::VERTICAL
        } else {
            ""
        };

        format!(
            "{}{}{}{}",
            boundary_type::VERTICAL,
            label,
            boundary_type::HORIZONTAL.repeat(right_padding),
            trailing_separator,
        )
    }

    fn stacked_pane_label(&self, title: &str, width: usize, is_selected: bool) -> String {
        if width == 0 {
            return String::new();
        }

        if is_selected {
            let truncated_title = truncate_text_to_width(title, width.saturating_sub(2));
            let wrapped_title = format!("[{}]", truncated_title);
            if wrapped_title.width() <= width {
                wrapped_title
            } else {
                self.truncate_to_width(&wrapped_title, width)
            }
        } else {
            let padded_title = format!(" {} ", title);
            if padded_title.width() <= width {
                padded_title
            } else {
                self.truncate_to_width(title, width)
            }
        }
    }

    fn truncate_to_width(&self, text: &str, width: usize) -> String {
        truncate_text_to_width(text, width)
    }

    pub fn render_pane_boundaries(
        &self,
        client_id: ClientId,
        client_mode: InputMode,
        boundaries: &mut Boundaries,
        session_is_mirrored: bool,
        pane_is_on_top_of_stack: bool,
        pane_is_on_bottom_of_stack: bool,
    ) {
        let color = self.frame_color(client_id, client_mode, session_is_mirrored);
        boundaries.add_rect(
            self.pane.as_ref(),
            color,
            pane_is_on_top_of_stack,
            pane_is_on_bottom_of_stack,
            self.pane_is_stacked_under,
        );
    }
    fn frame_color(
        &self,
        client_id: ClientId,
        mode: InputMode,
        session_is_mirrored: bool,
    ) -> Option<(PaletteColor, usize)> {
        // (color, color_precedence) (the color_precedence is used
        // for the no-pane-frames mode)
        let pane_focused_for_client_id = self.focused_clients.contains(&client_id);
        let pane_is_in_group = self
            .current_pane_group
            .get(&client_id)
            .map(|p| p.contains(&self.pane.pid()))
            .unwrap_or(false);
        if self.pane.frame_color_override().is_some() && !pane_is_in_group {
            self.pane
                .frame_color_override()
                .map(|override_color| (override_color, 4))
        } else if pane_is_in_group && !pane_focused_for_client_id {
            Some((self.style.colors.frame_highlight.emphasis_0, 2))
        } else if pane_is_in_group && pane_focused_for_client_id {
            Some((self.style.colors.frame_highlight.emphasis_1, 3))
        } else if pane_focused_for_client_id {
            match mode {
                InputMode::Normal | InputMode::Locked => {
                    if session_is_mirrored || !self.multiple_users_exist_in_session {
                        Some((self.style.colors.frame_selected.base, 3))
                    } else {
                        let colors = client_id_to_colors(
                            client_id,
                            self.style.colors.multiplayer_user_colors,
                        );
                        colors.map(|colors| (colors.0, 3))
                    }
                },
                _ => Some((self.style.colors.frame_highlight.base, 3)),
            }
        } else if self
            .mouse_is_hovering_over_pane_for_clients
            .contains(&client_id)
        {
            Some((self.style.colors.frame_highlight.base, 1))
        } else {
            self.style
                .colors
                .frame_unselected
                .map(|frame| (frame.base, 0))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use zellij_utils::data::{
        HeaderAlignment, PaneId as PluginPaneId, StackedPaneHeaderAction as PluginHeaderAction,
        StackedPaneHeaderItem, StackedPaneHeaderItemStyle, StackedPaneHeaderSpec,
    };

    fn stacked_header_with_width(cols: usize) -> StackedPaneHeader {
        let mut full_stack_geom = PaneGeom::default();
        full_stack_geom.cols.set_inner(cols);
        full_stack_geom.rows.set_inner(4);
        StackedPaneHeader {
            full_stack_geom,
            stack_id: 1,
            expanded_pane_id: PaneId::Terminal(1),
            tabs: vec![
                StackedPaneTab {
                    pane_id: PaneId::Terminal(1),
                    title: "pane-1".to_owned(),
                },
                StackedPaneTab {
                    pane_id: PaneId::Terminal(2),
                    title: "pane-2".to_owned(),
                },
            ],
        }
    }

    #[test]
    fn stacked_pane_header_segments_center_align_actions() {
        let stacked_pane_header = stacked_header_with_width(20);
        let spec = StackedPaneHeaderSpec {
            alignment: HeaderAlignment::Center,
            items: vec![
                StackedPaneHeaderItem {
                    pane_id: Some(PluginPaneId::Terminal(1)),
                    text: "one".to_owned(),
                    style: StackedPaneHeaderItemStyle::Default,
                    action: Some(PluginHeaderAction::FocusPane(PluginPaneId::Terminal(1))),
                },
                StackedPaneHeaderItem {
                    pane_id: Some(PluginPaneId::Terminal(2)),
                    text: "two".to_owned(),
                    style: StackedPaneHeaderItemStyle::Selected,
                    action: Some(PluginHeaderAction::ExpandPane(PluginPaneId::Terminal(2))),
                },
            ],
        };

        let segments = stacked_pane_header_segments(&stacked_pane_header, &spec);
        assert_eq!(segments.len(), 2);
        assert!(segments[0].start > 0);
        assert_eq!(segments[0].pane_id, Some(PaneId::Terminal(1)));
        assert_eq!(
            segments[1].action,
            Some(PluginHeaderAction::ExpandPane(PluginPaneId::Terminal(2)))
        );
    }

    #[test]
    fn stacked_pane_header_segments_truncate_to_inner_width() {
        let stacked_pane_header = stacked_header_with_width(8);
        let spec = StackedPaneHeaderSpec {
            alignment: HeaderAlignment::Left,
            items: vec![
                StackedPaneHeaderItem {
                    pane_id: Some(PluginPaneId::Terminal(1)),
                    text: "very-long".to_owned(),
                    style: StackedPaneHeaderItemStyle::Selected,
                    action: Some(PluginHeaderAction::FocusPane(PluginPaneId::Terminal(1))),
                },
                StackedPaneHeaderItem {
                    pane_id: Some(PluginPaneId::Terminal(2)),
                    text: "tail".to_owned(),
                    style: StackedPaneHeaderItemStyle::Default,
                    action: Some(PluginHeaderAction::FocusPane(PluginPaneId::Terminal(2))),
                },
            ],
        };

        let segments = stacked_pane_header_segments(&stacked_pane_header, &spec);
        let total_width: usize = segments.iter().map(|segment| segment.segment.width()).sum();
        assert!(total_width <= stacked_pane_header.full_stack_geom.cols.as_usize().saturating_sub(2));
        assert!(!segments.is_empty());
    }
}