use crate::docx::DocxDocument;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownDocument {
    pub elements: Vec<MarkdownElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownElement {
    Heading {
        level: u32,
        text: Vec<InlineElement>,
    },
    Paragraph {
        text: Vec<InlineElement>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    BlockQuote {
        elements: Vec<MarkdownElement>,
    },
    Table {
        headers: Vec<Vec<InlineElement>>,
        rows: Vec<Vec<Vec<InlineElement>>>,
    },
    HorizontalRule,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub elements: Vec<MarkdownElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InlineElement {
    Text(String),
    Bold(Vec<InlineElement>),
    Italic(Vec<InlineElement>),
    Code(String),
    Link {
        text: Vec<InlineElement>,
        url: String,
        title: Option<String>,
    },
    Image {
        alt: String,
        url: String,
        title: Option<String>,
    },
    LineBreak,
    SoftBreak,
}

impl Default for MarkdownDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownDocument {
    pub fn new() -> Self {
        MarkdownDocument {
            elements: Vec::new(),
        }
    }

    /// Parse markdown text into a MarkdownDocument
    pub fn from_markdown(markdown: &str) -> Result<Self> {
        // For now, use a simple line-by-line approach that works reliably
        let mut doc = MarkdownDocument::new();
        let lines: Vec<&str> = markdown.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            if line.is_empty() {
                i += 1;
                continue;
            }

            if line.starts_with('#') {
                // Heading
                let level = line.chars().take_while(|&c| c == '#').count() as u32;
                let text = line.trim_start_matches('#').trim();
                doc.elements.push(MarkdownElement::Heading {
                    level,
                    text: vec![InlineElement::Text(text.to_string())],
                });
            } else if line.starts_with("```") {
                // Code block
                let language = line.trim_start_matches('`').trim();
                let lang = if language.is_empty() {
                    None
                } else {
                    Some(language.to_string())
                };

                i += 1;
                let mut code = String::new();
                while i < lines.len() && !lines[i].trim().starts_with("```") {
                    if !code.is_empty() {
                        code.push('\n');
                    }
                    code.push_str(lines[i]);
                    i += 1;
                }

                doc.elements.push(MarkdownElement::CodeBlock {
                    language: lang,
                    code,
                });
            } else if line.starts_with('-') || line.starts_with('*') || line.starts_with('+') {
                // Unordered list
                let mut items = Vec::new();
                while i < lines.len()
                    && (lines[i].trim().starts_with('-')
                        || lines[i].trim().starts_with('*')
                        || lines[i].trim().starts_with('+'))
                {
                    let item_text = lines[i]
                        .trim()
                        .trim_start_matches(&['-', '*', '+'][..])
                        .trim();
                    items.push(ListItem {
                        elements: vec![MarkdownElement::Paragraph {
                            text: vec![InlineElement::Text(item_text.to_string())],
                        }],
                    });
                    i += 1;
                }
                i -= 1; // Adjust because the loop will increment

                doc.elements.push(MarkdownElement::List {
                    ordered: false,
                    items,
                });
            } else if line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains('.')
            {
                // Ordered list
                let mut items = Vec::new();
                while i < lines.len() {
                    let line = lines[i].trim();
                    if let Some(dot_pos) = line.find('.') {
                        if line[..dot_pos]
                            .chars()
                            .all(|c| c.is_ascii_digit() || c.is_whitespace())
                        {
                            let item_text = line[dot_pos + 1..].trim();
                            items.push(ListItem {
                                elements: vec![MarkdownElement::Paragraph {
                                    text: vec![InlineElement::Text(item_text.to_string())],
                                }],
                            });
                            i += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                i -= 1; // Adjust because the loop will increment

                doc.elements.push(MarkdownElement::List {
                    ordered: true,
                    items,
                });
            } else if line.starts_with('>') {
                // Blockquote
                let quote_text = line.trim_start_matches('>').trim();
                doc.elements.push(MarkdownElement::BlockQuote {
                    elements: vec![MarkdownElement::Paragraph {
                        text: vec![InlineElement::Text(quote_text.to_string())],
                    }],
                });
            } else if line == "---" || line == "***" || line == "___" {
                // Horizontal rule
                doc.elements.push(MarkdownElement::HorizontalRule);
            } else {
                // Regular paragraph
                doc.elements.push(MarkdownElement::Paragraph {
                    text: vec![InlineElement::Text(line.to_string())],
                });
            }

            i += 1;
        }

        Ok(doc)
    }

    /// Generate markdown text from the document
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&element.to_markdown());
        }

        output
    }

    /// Convert this MarkdownDocument to a DOCX document
    pub fn to_docx(&self) -> Result<DocxDocument> {
        let mut docx = DocxDocument::new();

        for element in &self.elements {
            match element {
                MarkdownElement::Heading { level, text } => {
                    let text_content = Self::inline_elements_to_text(text);
                    docx.add_heading(*level, &text_content)?;
                }
                MarkdownElement::Paragraph { text } => {
                    let text_content = Self::inline_elements_to_text(text);
                    docx.add_paragraph(&text_content)?;
                }
                MarkdownElement::CodeBlock { language: _, code } => {
                    docx.add_code_block(code)?;
                }
                MarkdownElement::List { ordered, items } => {
                    for (i, item) in items.iter().enumerate() {
                        for item_element in &item.elements {
                            match item_element {
                                MarkdownElement::Paragraph { text } => {
                                    let text_content = Self::inline_elements_to_text(text);
                                    if *ordered {
                                        docx.add_numbered_list_item(&text_content, i + 1)?;
                                    } else {
                                        docx.add_bulleted_list_item(&text_content)?;
                                    }
                                }
                                _ => {
                                    // Handle other list item content as needed
                                }
                            }
                        }
                    }
                }
                MarkdownElement::BlockQuote { elements } => {
                    for element in elements {
                        match element {
                            MarkdownElement::Paragraph { text } => {
                                let text_content = Self::inline_elements_to_text(text);
                                docx.add_quote(&text_content)?;
                            }
                            _ => {
                                // Handle other blockquote content as needed
                            }
                        }
                    }
                }
                MarkdownElement::Table {
                    headers: _,
                    rows: _,
                } => {
                    // Table support can be added later
                    docx.add_paragraph("[Table content - not yet supported in DOCX conversion]")?;
                }
                MarkdownElement::HorizontalRule => {
                    docx.add_horizontal_rule()?;
                }
            }
        }

        Ok(docx)
    }

    /// Convert a DOCX document to MarkdownDocument
    pub fn from_docx(docx: &DocxDocument) -> Result<Self> {
        // Parse the DOCX content and convert to markdown elements
        let mut doc = MarkdownDocument::new();

        // This is a simplified conversion - in a real implementation,
        // we would parse the DOCX XML structure more thoroughly
        let text = docx.get_text()?;
        let lines: Vec<&str> = text.lines().collect();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            // Simple heuristic to detect headings (this would be more sophisticated in practice)
            if Self::is_likely_heading(line) {
                doc.elements.push(MarkdownElement::Heading {
                    level: 1,
                    text: vec![InlineElement::Text(line.trim().to_string())],
                });
            } else {
                doc.elements.push(MarkdownElement::Paragraph {
                    text: vec![InlineElement::Text(line.trim().to_string())],
                });
            }
        }

        Ok(doc)
    }

    fn is_likely_heading(text: &str) -> bool {
        let text = text.trim();
        // Simple heuristics for detecting headings
        text.len() < 100
            && (text.chars().filter(|c| c.is_uppercase()).count() as f32 / text.len() as f32 > 0.3
                || text.ends_with(':')
                || !text.ends_with('.'))
    }

    fn inline_elements_to_text(elements: &[InlineElement]) -> String {
        let mut result = String::new();
        for element in elements {
            match element {
                InlineElement::Text(text) => result.push_str(text),
                InlineElement::Bold(inner) => {
                    result.push_str("**");
                    result.push_str(&Self::inline_elements_to_text(inner));
                    result.push_str("**");
                }
                InlineElement::Italic(inner) => {
                    result.push('*');
                    result.push_str(&Self::inline_elements_to_text(inner));
                    result.push('*');
                }
                InlineElement::Code(code) => {
                    result.push('`');
                    result.push_str(code);
                    result.push('`');
                }
                InlineElement::Link { text, url, .. } => {
                    result.push('[');
                    result.push_str(&Self::inline_elements_to_text(text));
                    result.push_str("](");
                    result.push_str(url);
                    result.push(')');
                }
                InlineElement::Image { alt, url, .. } => {
                    result.push_str("![");
                    result.push_str(alt);
                    result.push_str("](");
                    result.push_str(url);
                    result.push(')');
                }
                InlineElement::LineBreak => result.push_str("  \n"),
                InlineElement::SoftBreak => result.push(' '),
            }
        }
        result
    }
}

impl MarkdownElement {
    fn to_markdown(&self) -> String {
        match self {
            MarkdownElement::Heading { level, text } => {
                format!(
                    "{} {}\n",
                    "#".repeat(*level as usize),
                    Self::inline_elements_to_string(text)
                )
            }
            MarkdownElement::Paragraph { text } => {
                format!("{}\n", Self::inline_elements_to_string(text))
            }
            MarkdownElement::CodeBlock { language, code } => match language {
                Some(lang) => format!("```{}\n{}\n```\n", lang, code),
                None => format!("```\n{}\n```\n", code),
            },
            MarkdownElement::List { ordered, items } => {
                let mut result = String::new();
                for (i, item) in items.iter().enumerate() {
                    let prefix = if *ordered {
                        format!("{}. ", i + 1)
                    } else {
                        "- ".to_string()
                    };

                    for (j, element) in item.elements.iter().enumerate() {
                        if j == 0 {
                            result.push_str(&prefix);
                        } else {
                            result.push_str(&" ".repeat(prefix.len()));
                        }
                        result.push_str(element.to_markdown().trim_end());
                        result.push('\n');
                    }
                }
                result.push('\n');
                result
            }
            MarkdownElement::BlockQuote { elements } => {
                let mut result = String::new();
                for element in elements {
                    let element_md = element.to_markdown();
                    for line in element_md.lines() {
                        result.push_str("> ");
                        result.push_str(line);
                        result.push('\n');
                    }
                }
                result.push('\n');
                result
            }
            MarkdownElement::Table { headers, rows } => {
                let mut result = String::new();

                // Headers
                result.push('|');
                for header in headers {
                    result.push(' ');
                    result.push_str(&Self::inline_elements_to_string(header));
                    result.push_str(" |");
                }
                result.push('\n');

                // Separator
                result.push('|');
                for _ in headers {
                    result.push_str(" --- |");
                }
                result.push('\n');

                // Rows
                for row in rows {
                    result.push('|');
                    for cell in row {
                        result.push(' ');
                        result.push_str(&Self::inline_elements_to_string(cell));
                        result.push_str(" |");
                    }
                    result.push('\n');
                }

                result.push('\n');
                result
            }
            MarkdownElement::HorizontalRule => "---\n\n".to_string(),
        }
    }

    fn inline_elements_to_string(elements: &[InlineElement]) -> String {
        let mut result = String::new();
        for element in elements {
            match element {
                InlineElement::Text(text) => result.push_str(text),
                InlineElement::Bold(inner) => {
                    result.push_str("**");
                    result.push_str(&Self::inline_elements_to_string(inner));
                    result.push_str("**");
                }
                InlineElement::Italic(inner) => {
                    result.push('*');
                    result.push_str(&Self::inline_elements_to_string(inner));
                    result.push('*');
                }
                InlineElement::Code(code) => {
                    result.push('`');
                    result.push_str(code);
                    result.push('`');
                }
                InlineElement::Link { text, url, title } => {
                    result.push('[');
                    result.push_str(&Self::inline_elements_to_string(text));
                    result.push_str("](");
                    result.push_str(url);
                    if let Some(title) = title {
                        result.push_str(" \"");
                        result.push_str(title);
                        result.push('"');
                    }
                    result.push(')');
                }
                InlineElement::Image { alt, url, title } => {
                    result.push_str("![");
                    result.push_str(alt);
                    result.push_str("](");
                    result.push_str(url);
                    if let Some(title) = title {
                        result.push_str(" \"");
                        result.push_str(title);
                        result.push('"');
                    }
                    result.push(')');
                }
                InlineElement::LineBreak => result.push_str("  \n"),
                InlineElement::SoftBreak => result.push(' '),
            }
        }
        result
    }
}

use crate::OfficeDocument;
use std::path::Path;

impl OfficeDocument for MarkdownDocument {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized,
    {
        let content = std::fs::read_to_string(path)?;
        Self::from_markdown(&content)
    }

    fn from_bytes(data: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let content = String::from_utf8(data.to_vec()).map_err(|e| {
            crate::error::OfficeError::InvalidFormat(format!("Invalid UTF-8: {}", e))
        })?;
        Self::from_markdown(&content)
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let markdown_string = self.to_markdown();
        Ok(markdown_string.into_bytes())
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = self.to_markdown();
        std::fs::write(path, content)?;
        Ok(())
    }
}
