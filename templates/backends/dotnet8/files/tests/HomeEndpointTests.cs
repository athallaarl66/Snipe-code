using System.Net;
using Microsoft.AspNetCore.Mvc.Testing;
using Xunit;

public class HomeEndpointTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly WebApplicationFactory<Program> _factory;

    public HomeEndpointTests(WebApplicationFactory<Program> factory) => _factory = factory;

    [Fact]
    public async Task RootEndpointReturnsTheProductName()
    {
        var client = _factory.CreateClient();

        var response = await client.GetAsync("/");

        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
        Assert.Equal("SNIPE-CODE", await response.Content.ReadAsStringAsync());
    }
}
