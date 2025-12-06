use crate::application::ports::ExportService;
use crate::domain::models::{ColumnMetadata, ExportData};
use printpdf::*;
use std::sync::Arc;
use textwrap::{Options, WordSplitter};

// ============================================================================
// Font Management
// ============================================================================

/// Embedded font data - compiled into binary for portability
///
/// # Important: Font Path Limitations
///
/// The `include_bytes!` macro only accepts string literals, so paths cannot be
/// constructed dynamically using `CARGO_MANIFEST_DIR` or other environment variables.
///
/// These paths are relative to the source file location:
/// - `pdf.rs` is at `src/infrastructure/exporters/pdf.rs`
/// - Fonts are at `assets/fonts/`
/// - Relative path: `../../../assets/fonts/`
///
/// **If you move this file**, you must update these paths accordingly.
///
/// Alternative approaches for more flexibility:
/// 1. Use a `build.rs` script to generate font embedding code
/// 2. Load fonts at runtime from a configurable path
/// 3. Use the `include_dir` crate for directory-based embedding
mod embedded_fonts {
    pub const ANAKOTMAI_LIGHT: &[u8] = include_bytes!("../../../assets/fonts/Anakotmai-Light.ttf");
    pub const ANAKOTMAI_MEDIUM: &[u8] = include_bytes!("../../../assets/fonts/Anakotmai-Medium.ttf");
    pub const ANAKOTMAI_BOLD: &[u8] = include_bytes!("../../../assets/fonts/Anakotmai-Bold.ttf");
}

/// Font weight options
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontWeight {
    Light,
    #[default]
    Medium,
    Bold,
}

/// Font configuration for PDF generation
#[derive(Debug, Clone)]
pub struct FontConfig {
    pub regular_weight: FontWeight,
    pub bold_weight: FontWeight,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            regular_weight: FontWeight::Light,
            bold_weight: FontWeight::Bold,
        }
    }
}

/// Loaded fonts ready for PDF rendering
pub struct LoadedFonts {
    pub regular: IndirectFontRef,
    pub bold: IndirectFontRef,
}

/// Load fonts into a PDF document
///
/// # Arguments
/// * `doc` - Reference to the PDF document
/// * `config` - Font configuration specifying which weights to use
///
/// # Returns
/// * `Result<LoadedFonts, PdfExportError>` - Loaded font references or error
pub fn load_fonts(
    doc: &PdfDocumentReference,
    config: &FontConfig,
) -> Result<LoadedFonts, PdfExportError> {
    let regular_bytes = match config.regular_weight {
        FontWeight::Light => embedded_fonts::ANAKOTMAI_LIGHT,
        FontWeight::Medium => embedded_fonts::ANAKOTMAI_MEDIUM,
        FontWeight::Bold => embedded_fonts::ANAKOTMAI_BOLD,
    };

    let bold_bytes = match config.bold_weight {
        FontWeight::Light => embedded_fonts::ANAKOTMAI_LIGHT,
        FontWeight::Medium => embedded_fonts::ANAKOTMAI_MEDIUM,
        FontWeight::Bold => embedded_fonts::ANAKOTMAI_BOLD,
    };

    let regular = doc
        .add_external_font(regular_bytes)
        .map_err(|e| PdfExportError::FontLoading(format!("Regular font ({}): {}",
            format!("{:?}", config.regular_weight), e)))?;

    let bold = doc
        .add_external_font(bold_bytes)
        .map_err(|e| PdfExportError::FontLoading(format!("Bold font ({}): {}",
            format!("{:?}", config.bold_weight), e)))?;

    Ok(LoadedFonts { regular, bold })
}

/// Get raw font bytes by weight
#[allow(dead_code)]
pub fn get_font_bytes(weight: FontWeight) -> &'static [u8] {
    match weight {
        FontWeight::Light => embedded_fonts::ANAKOTMAI_LIGHT,
        FontWeight::Medium => embedded_fonts::ANAKOTMAI_MEDIUM,
        FontWeight::Bold => embedded_fonts::ANAKOTMAI_BOLD,
    }
}

// ============================================================================
// Domain Models for PDF (Value Objects)
// ============================================================================

/// PDF page size configuration
#[derive(Debug, Clone, Copy)]
pub struct PageSize {
    pub width: Mm,
    pub height: Mm,
}

impl PageSize {
    pub fn a4() -> Self {
        Self {
            width: Mm(210.0),
            height: Mm(297.0),
        }
    }

    pub fn letter() -> Self {
        Self {
            width: Mm(215.9),
            height: Mm(279.4),
        }
    }
}

impl Default for PageSize {
    fn default() -> Self {
        Self::a4()
    }
}

/// PDF margin configuration
#[derive(Debug, Clone, Copy)]
pub struct Margins {
    pub top: Mm,
    pub bottom: Mm,
    pub left: Mm,
    pub right: Mm,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: Mm(20.0),
            bottom: Mm(20.0),
            left: Mm(20.0),
            right: Mm(20.0),
        }
    }
}

/// Typography settings
#[derive(Debug, Clone, Copy)]
pub struct Typography {
    pub title_size: f32,
    pub header_size: f32,
    pub body_size: f32,
    pub page_number_size: f32,
    pub line_height: Mm,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            title_size: 16.0,
            header_size: 10.0,
            body_size: 10.0,
            page_number_size: 8.0,
            line_height: Mm(7.0),
        }
    }
}

/// Spacing configuration for PDF layout elements
#[derive(Debug, Clone, Copy)]
pub struct Spacing {
    /// Space below title before headers/content (mm)
    pub title_bottom: f32,
    /// Space between header text baseline and separator line (mm)
    pub header_line_offset: f32,
    /// Space between header line and first row (mm)
    pub header_to_content: f32,
    /// Padding inside cells (mm)
    pub cell_padding: f32,
    /// Space for page numbers from bottom (mm)
    pub page_number_area: f32,
    /// Offset for content start from top margin (mm)
    pub content_top_offset: f32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            title_bottom: 15.0,
            header_line_offset: 4.0,
            header_to_content: 10.0,
            cell_padding: 2.0,
            page_number_area: 20.0,
            content_top_offset: 10.0,
        }
    }
}

/// Complete PDF layout configuration
#[derive(Debug, Clone)]
pub struct PdfLayoutConfig {
    pub page_size: PageSize,
    pub margins: Margins,
    pub typography: Typography,
    pub spacing: Spacing,
    pub min_column_width: Mm,
    pub max_chars_per_cell: usize,
}

impl Default for PdfLayoutConfig {
    fn default() -> Self {
        Self {
            page_size: PageSize::default(),
            margins: Margins::default(),
            typography: Typography::default(),
            spacing: Spacing::default(),
            min_column_width: Mm(28.0),
            max_chars_per_cell: 50,
        }
    }
}

impl PdfLayoutConfig {
    /// Calculate available content width
    pub fn content_width(&self) -> Mm {
        Mm(self.page_size.width.0 - self.margins.left.0 - self.margins.right.0)
    }

    /// Calculate column width based on number of columns
    /// Always fits within page width - min_column_width is only a preference, not enforced
    pub fn calculate_column_width(&self, num_columns: usize) -> Mm {
        if num_columns == 0 {
            return self.min_column_width;
        }
        // Always divide evenly to fit within page, regardless of min_column_width
        Mm(self.content_width().0 / num_columns as f32)
    }

    /// Calculate starting Y position for content
    pub fn content_start_y(&self) -> Mm {
        Mm(self.page_size.height.0 - self.margins.top.0 - self.spacing.content_top_offset)
    }

    /// Calculate bottom margin with space for page numbers
    pub fn effective_bottom(&self) -> Mm {
        Mm(self.margins.bottom.0 + self.spacing.page_number_area)
    }
}

// ============================================================================
// Infrastructure Error Types
// ============================================================================

/// PDF-specific errors
#[derive(Debug)]
pub enum PdfExportError {
    FontLoading(String),
    Serialization(String),
}

impl std::fmt::Display for PdfExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FontLoading(msg) => write!(f, "Failed to load font: {}", msg),
            Self::Serialization(msg) => write!(f, "Failed to serialize PDF: {}", msg),
        }
    }
}

impl std::error::Error for PdfExportError {}

// ============================================================================
// Traits (Interfaces) - Single Responsibility
// ============================================================================

/// Responsible for text formatting and sanitization (SRP)
pub trait TextFormatter: Send + Sync {
    /// Sanitize text for PDF compatibility
    fn sanitize(&self, text: &str) -> String;

    /// Truncate text to fit within constraints
    fn truncate(&self, text: &str, max_chars: usize) -> String;

    /// Calculate max characters for given width and font size
    fn max_chars_for_width(&self, width_mm: f32, font_size: f32) -> usize;
}

// ============================================================================
// Implementations
// ============================================================================

/// Text truncation mode
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum TruncationMode {
    /// Simple character-based truncation (fast but may break words)
    Simple,
    /// Smart word-boundary aware truncation using textwrap
    WordBoundary,
}

impl Default for TruncationMode {
    fn default() -> Self {
        Self::WordBoundary
    }
}

/// Default text formatter with Latin character support and textwrap integration
pub struct LatinTextFormatter {
    max_chars_limit: usize,
    min_chars_limit: usize,
    truncation_mode: TruncationMode,
    ellipsis: String,
}

impl LatinTextFormatter {
    pub fn new() -> Self {
        Self {
            max_chars_limit: 50,
            min_chars_limit: 5,
            truncation_mode: TruncationMode::WordBoundary,
            ellipsis: "...".to_string(),
        }
    }

    /// Create formatter with custom ellipsis
    #[allow(dead_code)]
    pub fn with_ellipsis(mut self, ellipsis: &str) -> Self {
        self.ellipsis = ellipsis.to_string();
        self
    }

    /// Create formatter with specific truncation mode
    #[allow(dead_code)]
    pub fn with_truncation_mode(mut self, mode: TruncationMode) -> Self {
        self.truncation_mode = mode;
        self
    }

    /// Create formatter with custom character limits
    #[allow(dead_code)]
    pub fn with_limits(mut self, min: usize, max: usize) -> Self {
        self.min_chars_limit = min;
        self.max_chars_limit = max;
        self
    }

    /// Smart truncation using textwrap for proper word boundaries
    fn truncate_with_textwrap(&self, text: &str, max_chars: usize) -> String {
        let ellipsis_len = self.ellipsis.chars().count();

        if max_chars <= ellipsis_len {
            return text.chars().take(max_chars).collect();
        }

        // Calculate available width for text (minus ellipsis)
        let available_width = max_chars.saturating_sub(ellipsis_len);

        if available_width == 0 {
            return self.ellipsis.clone();
        }

        // Use textwrap to wrap at word boundaries
        let options = Options::new(available_width).word_splitter(WordSplitter::NoHyphenation);

        let wrapped = textwrap::wrap(text, options);

        if wrapped.is_empty() {
            return self.ellipsis.clone();
        }

        // Get the first line (fits within width)
        let first_line = wrapped[0].trim_end();

        // Check if we need ellipsis (text was truncated)
        if wrapped.len() > 1 || first_line.chars().count() < text.chars().count() {
            format!("{}{}", first_line, self.ellipsis)
        } else {
            first_line.to_string()
        }
    }

    /// Simple character-based truncation
    fn truncate_simple(&self, text: &str, max_chars: usize) -> String {
        let ellipsis_len = self.ellipsis.chars().count();

        if max_chars <= ellipsis_len {
            return text.chars().take(max_chars).collect();
        }

        let truncate_at = max_chars.saturating_sub(ellipsis_len);
        let truncated: String = text.chars().take(truncate_at).collect();

        format!("{}{}", truncated, self.ellipsis)
    }
}

impl Default for LatinTextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TextFormatter for LatinTextFormatter {
    fn sanitize(&self, text: &str) -> String {
        text.chars()
            .map(|c| match c {
                // ASCII printable characters
                '\u{0020}'..='\u{007E}' => c,
                // Thai characters (preserve them for Thai font support)
                '\u{0E00}'..='\u{0E7F}' => c,
                // Smart quotes -> regular quotes
                '\u{201C}' | '\u{201D}' => '"',
                '\u{2018}' | '\u{2019}' => '\'',
                // Dashes
                '\u{2013}' | '\u{2014}' => '-',
                // Ellipsis
                '\u{2026}' => '.',
                // Control characters -> space
                _ if c.is_ascii_control() => ' ',
                // Keep other Unicode characters (for multilingual support)
                _ => c,
            })
            .collect()
    }

    fn truncate(&self, text: &str, max_chars: usize) -> String {
        let char_count = text.chars().count();

        // No truncation needed
        if char_count <= max_chars {
            return text.to_string();
        }

        // Use appropriate truncation strategy
        match self.truncation_mode {
            TruncationMode::Simple => self.truncate_simple(text, max_chars),
            TruncationMode::WordBoundary => self.truncate_with_textwrap(text, max_chars),
        }
    }

    fn max_chars_for_width(&self, width_mm: f32, font_size: f32) -> usize {
        let width_pt = width_mm * 2.83465;
        let avg_char_width = font_size * 0.6;
        let max_chars = (width_pt / avg_char_width) as usize;
        max_chars.max(self.min_chars_limit).min(self.max_chars_limit)
    }
}

// ============================================================================
// PDF Document Builder (Builder Pattern)
// ============================================================================

/// Internal state for PDF page management
struct PageState {
    current_y: Mm,
    page_number: u32,
}

/// Column boundary coordinates for positioning
struct ColumnBounds {
    left: f32,
    right: f32,
}

/// PDF document renderer - focuses only on PDF rendering
struct PdfRenderer<'a> {
    doc: PdfDocumentReference,
    config: &'a PdfLayoutConfig,
    text_formatter: &'a dyn TextFormatter,
    font: IndirectFontRef,
    font_bold: IndirectFontRef,
    column_width: Mm,
}

impl<'a> PdfRenderer<'a> {
    fn new(
        title: &str,
        config: &'a PdfLayoutConfig,
        text_formatter: &'a dyn TextFormatter,
        num_columns: usize,
    ) -> Result<(Self, PdfPageIndex, PdfLayerIndex), PdfExportError> {
        Self::with_font_config(title, config, text_formatter, num_columns, &FontConfig::default())
    }

    fn with_font_config(
        title: &str,
        config: &'a PdfLayoutConfig,
        text_formatter: &'a dyn TextFormatter,
        num_columns: usize,
        font_config: &FontConfig,
    ) -> Result<(Self, PdfPageIndex, PdfLayerIndex), PdfExportError> {
        let sanitized_title = text_formatter.sanitize(title);
        let (doc, page_idx, layer_idx) = PdfDocument::new(
            &sanitized_title,
            config.page_size.width,
            config.page_size.height,
            "Layer 1",
        );

        // Load fonts using helper function
        let fonts = load_fonts(&doc, font_config)?;

        let column_width = config.calculate_column_width(num_columns);

        Ok((
            Self {
                doc,
                config,
                text_formatter,
                font: fonts.regular,
                font_bold: fonts.bold,
                column_width,
            },
            page_idx,
            layer_idx,
        ))
    }

    fn add_page(&self) -> (PdfPageIndex, PdfLayerIndex) {
        self.doc.add_page(
            self.config.page_size.width,
            self.config.page_size.height,
            "Layer 1",
        )
    }

    fn get_layer(&self, page_idx: PdfPageIndex, layer_idx: PdfLayerIndex) -> PdfLayerReference {
        self.doc.get_page(page_idx).get_layer(layer_idx)
    }

    fn render_title(&self, layer: &PdfLayerReference, title: &str, y: Mm) -> Mm {
        let sanitized = self.text_formatter.sanitize(title);
        layer.begin_text_section();
        layer.set_font(&self.font_bold, self.config.typography.title_size);
        layer.set_text_cursor(self.config.margins.left, y);
        layer.write_text(&sanitized, &self.font_bold);
        layer.end_text_section();
        Mm(y.0 - self.config.spacing.title_bottom)
    }

    fn render_headers(&self, layer: &PdfLayerReference, headers: &[String], y: Mm) -> Mm {
        // Each cell gets its own text section for proper absolute positioning
        for (col_idx, header) in headers.iter().enumerate() {
            layer.begin_text_section();
            layer.set_font(&self.font_bold, self.config.typography.header_size);

            // Sanitize header without truncation to preserve full header text
            let sanitized = self.text_formatter.sanitize(header);

            // Headers are always left-aligned
            let x_pos = Mm(self.config.margins.left.0 + self.column_width.0 * col_idx as f32);

            layer.set_text_cursor(x_pos, y);
            layer.write_text(&sanitized, &self.font_bold);
            layer.end_text_section();
        }

        // Position line below text baseline, with extra space for Thai descenders (สระล่าง)
        let line_y = Mm(y.0 - self.config.spacing.header_line_offset);
        self.render_header_line(layer, line_y);

        // Start next row below the line
        Mm(y.0 - self.config.spacing.header_to_content)
    }

    /// Check if a header represents numeric data
    fn is_numeric_header(header: &str) -> bool {
        let lower = header.to_lowercase();
        // Common numeric header patterns
        let numeric_keywords = [
            "amount", "total", "sum", "count", "qty", "quantity",
            "price", "cost", "rate", "value", "number", "num", "#",
            "balance", "credit", "debit", "fee", "tax", "discount",
            "percent", "%", "score", "points", "weight", "height",
            "width", "length", "size", "age", "year", "month", "day",
            "จำนวน", "ราคา", "รวม", "ยอด", "เงิน", "บาท",
        ];
        numeric_keywords.iter().any(|kw| lower.contains(kw))
    }

    /// Estimate text width in mm based on character count and font size
    fn estimate_text_width(text: &str, font_size: f32) -> f32 {
        let char_count = text.chars().count();
        // Average character width ratio for typical fonts
        let avg_char_width_pt = font_size * 0.5;
        let width_pt = char_count as f32 * avg_char_width_pt;
        // Convert points to mm (1 pt = 0.3528 mm)
        width_pt * 0.3528
    }

    fn render_header_line(&self, layer: &PdfLayerReference, y: Mm) {
        layer.set_outline_color(Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None)));
        layer.set_outline_thickness(0.5);
        let line = Line {
            points: vec![
                (Point::new(self.config.margins.left, y), false),
                (
                    Point::new(
                        Mm(self.config.page_size.width.0 - self.config.margins.right.0),
                        y,
                    ),
                    false,
                ),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }

    /// Calculate column boundaries for a given column index
    fn calculate_column_bounds(&self, col_idx: usize) -> ColumnBounds {
        let content_right = self.config.page_size.width.0 - self.config.margins.right.0;
        let left = self.config.margins.left.0 + self.column_width.0 * col_idx as f32;
        let right = (self.config.margins.left.0 + self.column_width.0 * (col_idx + 1) as f32)
            .min(content_right);
        ColumnBounds { left, right }
    }

    /// Determine if a column should be right-aligned based on metadata or header heuristic
    fn should_right_align(
        &self,
        col_idx: usize,
        headers: &[String],
        column_metadata: Option<&[ColumnMetadata]>,
    ) -> bool {
        // Priority 1: Use explicit column metadata if available
        if let Some(metadata) = column_metadata {
            if let Some(col_meta) = metadata.get(col_idx) {
                return col_meta.column_type.is_right_aligned();
            }
        }
        // Priority 2: Fall back to header-based heuristic
        headers
            .get(col_idx)
            .map(|h| Self::is_numeric_header(h))
            .unwrap_or(false)
    }

    /// Calculate x position for text based on alignment
    fn calculate_text_position(
        &self,
        text: &str,
        bounds: &ColumnBounds,
        right_align: bool,
    ) -> Mm {
        if right_align {
            let text_width = Self::estimate_text_width(text, self.config.typography.body_size);
            let right_aligned_x = bounds.right - text_width - self.config.spacing.cell_padding;
            Mm(right_aligned_x.max(bounds.left))
        } else {
            Mm(bounds.left)
        }
    }

    /// Prepare cell text: truncate and sanitize
    fn prepare_cell_text(&self, cell: &str) -> String {
        let max_chars = self
            .text_formatter
            .max_chars_for_width(self.column_width.0, self.config.typography.body_size);
        let truncated = self.text_formatter.truncate(cell, max_chars);
        self.text_formatter.sanitize(&truncated)
    }

    /// Render a single cell at the specified position
    fn render_cell(&self, layer: &PdfLayerReference, text: &str, x: Mm, y: Mm) {
        layer.begin_text_section();
        layer.set_font(&self.font, self.config.typography.body_size);
        layer.set_text_cursor(x, y);
        layer.write_text(text, &self.font);
        layer.end_text_section();
    }

    /// Render a complete data row
    fn render_row(
        &self,
        layer: &PdfLayerReference,
        row: &[String],
        headers: &[String],
        column_metadata: Option<&[ColumnMetadata]>,
        y: Mm,
    ) {
        for (col_idx, cell) in row.iter().enumerate() {
            let sanitized = self.prepare_cell_text(cell);
            let bounds = self.calculate_column_bounds(col_idx);
            let right_align = self.should_right_align(col_idx, headers, column_metadata);
            let x_pos = self.calculate_text_position(&sanitized, &bounds, right_align);
            self.render_cell(layer, &sanitized, x_pos, y);
        }
    }

    fn render_page_number(&self, layer: &PdfLayerReference, page_num: u32) {
        layer.begin_text_section();
        layer.set_font(&self.font, self.config.typography.page_number_size);
        layer.set_text_cursor(
            Mm(self.config.page_size.width.0 / 2.0 - 10.0),
            self.config.margins.bottom,
        );
        layer.write_text(&format!("Page {}", page_num), &self.font);
        layer.end_text_section();
    }

    fn save_to_bytes(self) -> Result<Vec<u8>, PdfExportError> {
        self.doc
            .save_to_bytes()
            .map_err(|e| PdfExportError::Serialization(e.to_string()))
    }
}

// ============================================================================
// Main PDF Exporter (Dependency Injection)
// ============================================================================

/// PDF exporter with configurable dependencies
pub struct PdfExporter {
    config: PdfLayoutConfig,
    text_formatter: Arc<dyn TextFormatter>,
}

impl PdfExporter {
    /// Create with default configuration
    pub fn new() -> Self {
        Self {
            config: PdfLayoutConfig::default(),
            text_formatter: Arc::new(LatinTextFormatter::new()),
        }
    }

    /// Create with custom configuration (Open/Closed Principle)
    pub fn with_config(config: PdfLayoutConfig) -> Self {
        Self {
            config,
            text_formatter: Arc::new(LatinTextFormatter::new()),
        }
    }

    /// Create with custom text formatter (Dependency Inversion)
    pub fn with_formatter(text_formatter: Arc<dyn TextFormatter>) -> Self {
        Self {
            config: PdfLayoutConfig::default(),
            text_formatter,
        }
    }

    /// Builder-style configuration
    pub fn config(mut self, config: PdfLayoutConfig) -> Self {
        self.config = config;
        self
    }

    /// Builder-style text formatter injection
    pub fn formatter(mut self, formatter: Arc<dyn TextFormatter>) -> Self {
        self.text_formatter = formatter;
        self
    }
}

impl Default for PdfExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportService for PdfExporter {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let (renderer, mut page_idx, mut layer_idx) = PdfRenderer::new(
            &data.title,
            &self.config,
            self.text_formatter.as_ref(),
            data.headers.len(),
        )?;

        let mut state = PageState {
            current_y: self.config.content_start_y(),
            page_number: 1,
        };

        let mut layer = renderer.get_layer(page_idx, layer_idx);

        // Render title
        state.current_y = renderer.render_title(&layer, &data.title, state.current_y);

        // Render headers on first page
        if !data.headers.is_empty() {
            state.current_y = renderer.render_headers(&layer, &data.headers, state.current_y);
        }

        // Render data rows with pagination
        for row in &data.rows {
            if state.current_y < self.config.effective_bottom() {
                renderer.render_page_number(&layer, state.page_number);

                state.page_number += 1;
                let (new_page_idx, new_layer_idx) = renderer.add_page();
                page_idx = new_page_idx;
                layer_idx = new_layer_idx;
                layer = renderer.get_layer(page_idx, layer_idx);

                state.current_y = self.config.content_start_y();

                if !data.headers.is_empty() {
                    state.current_y =
                        renderer.render_headers(&layer, &data.headers, state.current_y);
                }
            }

            renderer.render_row(&layer, row, &data.headers, data.column_metadata.as_deref(), state.current_y);
            state.current_y = Mm(state.current_y.0 - self.config.typography.line_height.0);
        }

        renderer.render_page_number(&layer, state.page_number);

        renderer.save_to_bytes().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ColumnType;

    #[test]
    fn test_page_size_a4() {
        let size = PageSize::a4();
        assert!((size.width.0 - 210.0).abs() < f32::EPSILON);
        assert!((size.height.0 - 297.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_page_size_letter() {
        let size = PageSize::letter();
        assert!((size.width.0 - 215.9).abs() < f32::EPSILON);
        assert!((size.height.0 - 279.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_layout_config_content_width() {
        let config = PdfLayoutConfig::default();
        let width = config.content_width();
        // 210 - 20 - 20 = 170
        assert!((width.0 - 170.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_layout_config_column_width_calculation() {
        let config = PdfLayoutConfig::default();

        // 5 columns: 170 / 5 = 34mm
        let width = config.calculate_column_width(5);
        assert!((width.0 - 34.0).abs() < f32::EPSILON);

        // 0 columns: should return min width
        let width = config.calculate_column_width(0);
        assert!((width.0 - 28.0).abs() < f32::EPSILON);

        // 10 columns: 170 / 10 = 17mm (always fit within page)
        let width = config.calculate_column_width(10);
        assert!((width.0 - 17.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_text_formatter_sanitize() {
        let formatter = LatinTextFormatter::new();

        assert_eq!(formatter.sanitize("Hello World"), "Hello World");
        assert_eq!(formatter.sanitize("Test\u{201C}Quote\u{201D}"), "Test\"Quote\"");
        // Thai characters are now preserved (not replaced with ?)
        assert_eq!(formatter.sanitize("Thai: \u{0E01}"), "Thai: \u{0E01}");
        // Full Thai text preserved
        assert_eq!(formatter.sanitize("สวัสดี"), "สวัสดี");
    }

    #[test]
    fn test_text_formatter_truncate_word_boundary() {
        let formatter = LatinTextFormatter::new();

        // Text shorter than limit - no truncation
        assert_eq!(formatter.truncate("Short", 10), "Short");

        // Text at exact limit - no truncation
        assert_eq!(formatter.truncate("TenCharStr", 10), "TenCharStr");

        // Very short limit - just take characters
        assert_eq!(formatter.truncate("AB", 2), "AB");

        // Word boundary truncation with textwrap:
        // max 15 chars, ellipsis "..." = 3, available = 12
        // "This is a test" wraps to first line that fits
        let result = formatter.truncate("This is a test sentence", 15);
        assert!(result.ends_with("..."), "Result should end with ellipsis: {}", result);
        assert!(result.chars().count() <= 15, "Result too long: {}", result);

        // Longer text with multiple words
        let result = formatter.truncate("The quick brown fox jumps over the lazy dog", 20);
        assert!(result.ends_with("..."));
        assert!(result.chars().count() <= 20);
    }

    #[test]
    fn test_text_formatter_truncate_simple_mode() {
        let formatter = LatinTextFormatter::new()
            .with_truncation_mode(TruncationMode::Simple);

        // Simple mode just cuts at character boundary
        let result = formatter.truncate("Hello World Test", 10);
        assert_eq!(result, "Hello W...");

        // No truncation needed
        assert_eq!(formatter.truncate("Short", 10), "Short");
    }

    #[test]
    fn test_text_formatter_custom_ellipsis() {
        let formatter = LatinTextFormatter::new()
            .with_ellipsis("…")
            .with_truncation_mode(TruncationMode::Simple);

        let result = formatter.truncate("Hello World", 8);
        assert!(result.ends_with("…"));
        // "Hello W" + "…" = 8 chars
        assert_eq!(result.chars().count(), 8);
    }

    #[test]
    fn test_text_formatter_no_spaces_word_boundary() {
        let formatter = LatinTextFormatter::new();

        // When there are no spaces, textwrap will keep the word intact
        // if it fits, otherwise it may break it
        let result = formatter.truncate("NoSpacesHereAtAll", 10);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_text_formatter_max_chars() {
        let formatter = LatinTextFormatter::new();

        let chars = formatter.max_chars_for_width(28.0, 10.0);
        assert!(chars >= 5 && chars <= 50);
    }

    #[test]
    fn test_pdf_exporter_creation() {
        let exporter = PdfExporter::new();
        assert!((exporter.config.page_size.width.0 - 210.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pdf_exporter_with_custom_config() {
        let config = PdfLayoutConfig {
            page_size: PageSize::letter(),
            ..Default::default()
        };
        let exporter = PdfExporter::with_config(config);
        assert!((exporter.config.page_size.width.0 - 215.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pdf_export_basic() {
        let exporter = PdfExporter::new();
        let data = ExportData {
            title: "Test Report".to_string(),
            format: crate::domain::models::ExportFormat::Pdf,
            headers: vec!["Name".to_string(), "Value".to_string()],
            rows: vec![
                vec!["Item 1".to_string(), "100".to_string()],
                vec!["Item 2".to_string(), "200".to_string()],
            ],
            options: None,
            column_metadata: None,
        };

        let result = exporter.export(&data);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        // PDF files start with %PDF
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_is_numeric_header() {
        // English numeric keywords
        assert!(PdfRenderer::is_numeric_header("Amount"));
        assert!(PdfRenderer::is_numeric_header("Total Sales"));
        assert!(PdfRenderer::is_numeric_header("Quantity"));
        assert!(PdfRenderer::is_numeric_header("Price"));
        assert!(PdfRenderer::is_numeric_header("Item Count"));
        assert!(PdfRenderer::is_numeric_header("Discount %"));
        assert!(PdfRenderer::is_numeric_header("Score"));

        // Thai numeric keywords
        assert!(PdfRenderer::is_numeric_header("จำนวน"));
        assert!(PdfRenderer::is_numeric_header("ราคาสินค้า"));
        assert!(PdfRenderer::is_numeric_header("ยอดรวม"));

        // Non-numeric headers
        assert!(!PdfRenderer::is_numeric_header("Name"));
        assert!(!PdfRenderer::is_numeric_header("Description"));
        assert!(!PdfRenderer::is_numeric_header("Status"));
        assert!(!PdfRenderer::is_numeric_header("ชื่อ"));
    }

    #[test]
    fn test_estimate_text_width() {
        // Width should scale with text length
        let short_width = PdfRenderer::estimate_text_width("Hi", 10.0);
        let long_width = PdfRenderer::estimate_text_width("Hello World", 10.0);
        assert!(long_width > short_width);

        // Width should scale with font size
        let small_font = PdfRenderer::estimate_text_width("Test", 8.0);
        let large_font = PdfRenderer::estimate_text_width("Test", 16.0);
        assert!(large_font > small_font);

        // Verify approximate calculation (4 chars * 10pt * 0.5 * 0.3528 = 7.056mm)
        let width = PdfRenderer::estimate_text_width("Test", 10.0);
        assert!((width - 7.056).abs() < 0.1);
    }

    #[test]
    fn test_pdf_export_with_numeric_headers() {
        let exporter = PdfExporter::new();
        let data = ExportData {
            title: "Sales Report".to_string(),
            format: crate::domain::models::ExportFormat::Pdf,
            headers: vec![
                "Product Name".to_string(),
                "Quantity".to_string(),
                "Price".to_string(),
                "Total Amount".to_string(),
            ],
            rows: vec![
                vec!["Widget A".to_string(), "10".to_string(), "99.99".to_string(), "999.90".to_string()],
                vec!["Widget B".to_string(), "5".to_string(), "149.99".to_string(), "749.95".to_string()],
            ],
            options: None,
            column_metadata: None,
        };

        let result = exporter.export(&data);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_pdf_export_with_explicit_column_metadata() {
        let exporter = PdfExporter::new();
        let data = ExportData {
            title: "Metadata Test".to_string(),
            format: crate::domain::models::ExportFormat::Pdf,
            headers: vec![
                "Code".to_string(),      // Looks like text but we want right-align
                "Description".to_string(), // Text, left-align
                "Value".to_string(),     // Looks numeric but we want left-align
            ],
            rows: vec![
                vec!["001".to_string(), "Item A".to_string(), "100".to_string()],
                vec!["002".to_string(), "Item B".to_string(), "200".to_string()],
            ],
            options: None,
            column_metadata: Some(vec![
                ColumnMetadata::number(),    // Code: right-aligned
                ColumnMetadata::text(),      // Description: left-aligned
                ColumnMetadata::text(),      // Value: left-aligned (override heuristic)
            ]),
        };

        let result = exporter.export(&data);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_column_type_alignment() {
        assert!(!ColumnType::Text.is_right_aligned());
        assert!(!ColumnType::Date.is_right_aligned());
        assert!(ColumnType::Number.is_right_aligned());
        assert!(ColumnType::Currency.is_right_aligned());
        assert!(ColumnType::Percentage.is_right_aligned());
    }

    #[test]
    fn test_column_metadata_builders() {
        let text = ColumnMetadata::text();
        assert_eq!(text.column_type, ColumnType::Text);
        assert!(text.width_hint.is_none());

        let number_with_width = ColumnMetadata::number().with_width(50.0);
        assert_eq!(number_with_width.column_type, ColumnType::Number);
        assert_eq!(number_with_width.width_hint, Some(50.0));
    }
}
