using System.IO.Compression;
using System.Text.Json;
using DocumentFormat.OpenXml;
using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Validation;

if (args.Length == 0)
{
    Console.Error.WriteLine("Usage: dotnet run -- FILE.pptx [FILE.pptx ...]");
    return 2;
}

var failures = new List<object>();
var validator = new OpenXmlValidator(FileFormatVersions.Microsoft365) { MaxNumberOfErrors = 0 };
void Validate(OpenXmlPackage package, string source)
{
    foreach (var error in validator.Validate(package))
        failures.Add(new {
            Source = source,
            error.Id,
            Type = error.ErrorType.ToString(),
            error.Description,
            Part = error.Part?.Uri.ToString(),
            Path = error.Path?.XPath
        });
}

foreach (var file in args)
{
    using (var deck = PresentationDocument.Open(file, false)) Validate(deck, Path.GetFileName(file));
    using var archive = ZipFile.OpenRead(file);
    foreach (var entry in archive.Entries.Where(e => e.FullName.StartsWith("ppt/embeddings/") && e.FullName.EndsWith(".xlsx")))
    {
        using var input = entry.Open();
        using var memory = new MemoryStream();
        input.CopyTo(memory);
        memory.Position = 0;
        using var workbook = SpreadsheetDocument.Open(memory, false);
        Validate(workbook, entry.FullName);
    }
}

Console.WriteLine(JsonSerializer.Serialize(new { ErrorCount = failures.Count, Errors = failures }, new JsonSerializerOptions { WriteIndented = true }));
return failures.Count == 0 ? 0 : 1;
