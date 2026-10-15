var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();

var app = builder.Build();

app.MapControllers();
app.MapGet("/", () => "SNIPE-CODE");

app.Run("http://0.0.0.0:5000");

// Exposed so the xunit project can drive the app through WebApplicationFactory.
public partial class Program { }
