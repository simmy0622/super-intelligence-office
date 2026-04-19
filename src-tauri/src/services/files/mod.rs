use std::{
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use calamine::Reader;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use quick_xml::{events::Event, Reader as XmlReader};
use rust_xlsxwriter::Workbook;
use serde_json::Value;
use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

pub const MAX_UPLOAD_BYTES: usize = 32 * 1024 * 1024;

pub fn uploads_dir(app_data_dir: &Path) -> Result<PathBuf, String> {
    let dir = app_data_dir.join("uploads");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

pub fn new_storage_name(original_name: &str, kind: &str) -> String {
    let extension = extension_for_kind(kind)
        .or_else(|| {
            Path::new(original_name)
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase())
        })
        .unwrap_or_else(|| "bin".to_string());
    format!("{}.{}", Uuid::new_v4(), extension)
}

pub fn kind_from_filename(name: &str) -> Option<String> {
    let ext = Path::new(name)
        .extension()
        .and_then(|value| value.to_str())?
        .to_ascii_lowercase();
    match ext.as_str() {
        "pdf" => Some("pdf".to_string()),
        "docx" => Some("docx".to_string()),
        "pptx" => Some("pptx".to_string()),
        "xlsx" | "xls" => Some("xlsx".to_string()),
        "csv" => Some("csv".to_string()),
        "png" | "jpg" | "jpeg" | "webp" => Some("image".to_string()),
        "md" | "markdown" => Some("md".to_string()),
        _ => None,
    }
}

pub fn extension_for_kind(kind: &str) -> Option<String> {
    match kind {
        "pdf" => Some("pdf".to_string()),
        "docx" => Some("docx".to_string()),
        "pptx" => Some("pptx".to_string()),
        "xlsx" => Some("xlsx".to_string()),
        "csv" => Some("csv".to_string()),
        "image" => Some("jpg".to_string()),
        "md" => Some("md".to_string()),
        _ => None,
    }
}

pub fn extract_text(path: &Path, kind: &str) -> Option<String> {
    let text = match kind {
        "pdf" => pdf_extract::extract_text(path).ok()?,
        "docx" => extract_docx_text(path).ok()?,
        "pptx" => extract_pptx_text(path).ok()?,
        "xlsx" => extract_xlsx_text(path).ok()?,
        "csv" => extract_csv_text(path).ok()?,
        "md" => fs::read_to_string(path).ok()?,
        "image" => return None,
        _ => return None,
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.chars().take(200_000).collect())
    }
}

pub fn generate_file(format: &str, content: &str, dest: &Path) -> Result<(), String> {
    match format {
        "md" | "csv" => fs::write(dest, content).map_err(|error| error.to_string()),
        "docx" => generate_docx(content, dest),
        "xlsx" => generate_xlsx(content, dest),
        "pdf" => generate_pdf(content, dest),
        "pptx" => generate_pptx(content, dest),
        other => Err(format!("unsupported file format: {other}")),
    }
}

fn extract_docx_text(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    let mut document = archive
        .by_name("word/document.xml")
        .map_err(|error| error.to_string())?;
    let mut xml = String::new();
    document
        .read_to_string(&mut xml)
        .map_err(|error| error.to_string())?;
    extract_xml_text(&xml)
}

fn extract_pptx_text(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    let mut names = (0..archive.len())
        .filter_map(|index| archive.by_index(index).ok().map(|file| file.name().to_string()))
        .filter(|name| name.starts_with("ppt/slides/slide") && name.ends_with(".xml"))
        .collect::<Vec<_>>();
    names.sort();

    let mut output = Vec::new();
    for name in names {
        let mut slide = archive.by_name(&name).map_err(|error| error.to_string())?;
        let mut xml = String::new();
        slide
            .read_to_string(&mut xml)
            .map_err(|error| error.to_string())?;
        let text = extract_xml_text(&xml)?;
        if !text.trim().is_empty() {
            output.push(text);
        }
    }
    Ok(output.join("\n\n"))
}

fn extract_xml_text(xml: &str) -> Result<String, String> {
    let mut reader = XmlReader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut text = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(value)) => {
                let value = value.unescape().map_err(|error| error.to_string())?;
                let value = value.trim();
                if !value.is_empty() {
                    text.push(value.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(error.to_string()),
        }
        buf.clear();
    }
    Ok(text.join("\n"))
}

fn extract_xlsx_text(path: &Path) -> Result<String, String> {
    let mut workbook = calamine::open_workbook_auto(path).map_err(|error| error.to_string())?;
    let mut output = Vec::new();
    for sheet_name in workbook.sheet_names().to_owned() {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            output.push(format!("# Sheet: {sheet_name}"));
            for row in range.rows().take(2000) {
                let cells = row.iter().map(|cell| cell.to_string()).collect::<Vec<_>>();
                if cells.iter().any(|cell| !cell.trim().is_empty()) {
                    output.push(cells.join("\t"));
                }
            }
        }
    }
    Ok(output.join("\n"))
}

fn extract_csv_text(path: &Path) -> Result<String, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for record in reader.records().take(5000) {
        rows.push(record.map_err(|error| error.to_string())?.iter().collect::<Vec<_>>().join(", "));
    }
    Ok(rows.join("\n"))
}

fn generate_docx(content: &str, dest: &Path) -> Result<(), String> {
    use docx_rs::{Docx, Paragraph, Run};

    let mut doc = Docx::new();
    for line in content.lines() {
        doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line)));
    }
    let file = File::create(dest).map_err(|error| error.to_string())?;
    doc.build().pack(file).map_err(|error| error.to_string())
}

fn generate_xlsx(content: &str, dest: &Path) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    if let Ok(Value::Array(rows)) = serde_json::from_str::<Value>(content) {
        for (row_index, row_value) in rows.iter().enumerate() {
            let row_values = row_value.as_array().cloned().unwrap_or_else(|| vec![row_value.clone()]);
            for (col_index, value) in row_values.iter().enumerate() {
                worksheet
                    .write_string(row_index as u32, col_index as u16, value_to_cell(value))
                    .map_err(|error| error.to_string())?;
            }
        }
    } else {
        for (row_index, line) in content.lines().enumerate() {
            for (col_index, cell) in line.split(',').enumerate() {
                worksheet
                    .write_string(row_index as u32, col_index as u16, cell.trim())
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    workbook.save(dest).map_err(|error| error.to_string())
}

fn value_to_cell(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn generate_pdf(content: &str, dest: &Path) -> Result<(), String> {
    let (doc, page, layer) = PdfDocument::new("Agent Salon Export", Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(page).get_layer(layer);
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|error| error.to_string())?;
    let mut y = 277.0;
    for line in content.lines().flat_map(wrap_pdf_line).take(55) {
        layer.use_text(line, 11.0, Mm(18.0), Mm(y), &font);
        y -= 5.0;
    }
    let file = File::create(dest).map_err(|error| error.to_string())?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|error| error.to_string())
}

fn wrap_pdf_line(line: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for ch in line.chars() {
        current.push(ch);
        if current.chars().count() >= 88 {
            lines.push(current);
            current = String::new();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn generate_pptx(content: &str, dest: &Path) -> Result<(), String> {
    let slides = parse_slides(content);
    let file = File::create(dest).map_err(|error| error.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("[Content_Types].xml", options).map_err(|error| error.to_string())?;
    zip.write_all(content_types_xml(slides.len()).as_bytes()).map_err(|error| error.to_string())?;
    zip.start_file("_rels/.rels", options).map_err(|error| error.to_string())?;
    zip.write_all(root_rels_xml().as_bytes()).map_err(|error| error.to_string())?;
    zip.start_file("ppt/presentation.xml", options).map_err(|error| error.to_string())?;
    zip.write_all(presentation_xml(slides.len()).as_bytes()).map_err(|error| error.to_string())?;
    zip.start_file("ppt/_rels/presentation.xml.rels", options).map_err(|error| error.to_string())?;
    zip.write_all(presentation_rels_xml(slides.len()).as_bytes()).map_err(|error| error.to_string())?;

    for (index, slide) in slides.iter().enumerate() {
        zip.start_file(format!("ppt/slides/slide{}.xml", index + 1), options)
            .map_err(|error| error.to_string())?;
        zip.write_all(slide_xml(&slide.title, &slide.bullets).as_bytes())
            .map_err(|error| error.to_string())?;
    }
    zip.finish().map_err(|error| error.to_string())?;
    Ok(())
}

#[derive(Debug, Clone)]
struct SlideSpec {
    title: String,
    bullets: Vec<String>,
}

fn parse_slides(content: &str) -> Vec<SlideSpec> {
    if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(content) {
        let slides = items
            .into_iter()
            .filter_map(|item| {
                let title = item.get("title").and_then(Value::as_str)?.to_string();
                let bullets = item
                    .get("bullets")
                    .and_then(Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                Some(SlideSpec { title, bullets })
            })
            .collect::<Vec<_>>();
        if !slides.is_empty() {
            return slides;
        }
    }

    content
        .split("\n---")
        .map(|chunk| {
            let mut lines = chunk.lines().filter(|line| !line.trim().is_empty());
            let title = lines.next().unwrap_or("Untitled").trim().trim_start_matches('#').trim().to_string();
            let bullets = lines
                .map(|line| line.trim().trim_start_matches('-').trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            SlideSpec { title, bullets }
        })
        .filter(|slide| !slide.title.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .take(20)
        .collect()
}

fn content_types_xml(slide_count: usize) -> String {
    let overrides = (1..=slide_count)
        .map(|index| {
            format!(
                r#"<Override PartName="/ppt/slides/slide{index}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#
            )
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
{overrides}
</Types>"#
    )
}

fn root_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#
        .to_string()
}

fn presentation_xml(slide_count: usize) -> String {
    let ids = (1..=slide_count)
        .map(|index| format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + index, index))
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:sldIdLst>{ids}</p:sldIdLst>
<p:sldSz cx="9144000" cy="5143500" type="screen16x9"/>
<p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#
    )
}

fn presentation_rels_xml(slide_count: usize) -> String {
    let rels = (1..=slide_count)
        .map(|index| {
            format!(
                r#"<Relationship Id="rId{index}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{index}.xml"/>"#
            )
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{rels}</Relationships>"#
    )
}

fn slide_xml(title: &str, bullets: &[String]) -> String {
    let bullet_text = bullets
        .iter()
        .take(8)
        .enumerate()
        .map(|(index, bullet)| {
            let y = 1_550_000 + (index as i64 * 420_000);
            text_shape_xml(3 + index, 700_000, y, 7_700_000, 340_000, &format!("• {bullet}"), 2400)
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
{}
{bullet_text}
</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>"#,
        text_shape_xml(2, 600_000, 500_000, 7_900_000, 650_000, title, 3600)
    )
}

fn text_shape_xml(id: usize, x: i64, y: i64, cx: i64, cy: i64, text: &str, size: i64) -> String {
    format!(
        r#"<p:sp><p:nvSpPr><p:cNvPr id="{id}" name="Text {id}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/></p:spPr><p:txBody><a:bodyPr wrap="square"/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="{size}"/><a:t>{}</a:t></a:r></a:p></p:txBody></p:sp>"#,
        escape_xml(text)
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
