# Report font

`NotoSans-Regular.ttf` and `OFL.txt` come from
[notofonts/noto-fonts at ffebf8c1ee449e544955a7e813c54f9b73848eac](https://github.com/notofonts/noto-fonts/tree/ffebf8c1ee449e544955a7e813c54f9b73848eac).
The font path is `hinted/ttf/NotoSans/NotoSans-Regular.ttf`; the upstream
license is SIL Open Font License 1.1. The file is unmodified.

The report renderer embeds this font in PDF and HTML and uses its actual
advance widths for pagination. PowerPoint text specifies the font family;
PowerPoint/LibreOffice may substitute if it is not installed. A caller may
supply another licensed font via `ReportRenderOptions.font_data`. Missing
glyphs fail explicitly; the current renderer does not shape complex scripts.
