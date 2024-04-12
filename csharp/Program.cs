using System.Text;
using System.Text.Json;
using System.Security.Cryptography;
using System.Text.Json.Serialization;
using System.Net.Http.Json;


struct CoinExHttpResp<T>
{
    [JsonPropertyName("code")]
    public int Code { get; set; }

    [JsonPropertyName("message")]
    public string Message { get; set; }

    [JsonPropertyName("data")]
    public T Data { get; set; }

    public void EnsureSuccessStatusCode()
    {
        if (Code != 0)
        {
            throw new HttpRequestException($"{Code}: {Message}");
        }
    }
}


struct Order
{
    [JsonPropertyName("order_id")]
    public long OrderId { get; set; }

    [JsonPropertyName("market")]
    public string Market { get; set; }

    [JsonPropertyName("market_type")]
    public string MarketType { get; set; }

    [JsonPropertyName("ccy")]
    public string Currency { get; set; }

    [JsonPropertyName("side")]
    public string Side { get; set; }

    [JsonPropertyName("type")]
    public string Type { get; set; }

    [JsonPropertyName("amount")]
    public string Amount { get; set; }

    [JsonPropertyName("price")]
    public string Price { get; set; }

    [JsonPropertyName("unfilled_amount")]
    public string UnfilledAmount { get; set; }

    [JsonPropertyName("filled_amount")]
    public string FilledAmount { get; set; }

    [JsonPropertyName("filled_value")]
    public string FilledValue { get; set; }

    [JsonPropertyName("client_id")]
    public string ClientId { get; set; }

    [JsonPropertyName("base_fee")]
    public string BaseFee { get; set; }

    [JsonPropertyName("quote_fee")]
    public string QuoteFee { get; set; }

    [JsonPropertyName("discount_fee")]
    public string DiscountFee { get; set; }

    [JsonPropertyName("maker_fee_rate")]
    public string MakerFeeRate { get; set; }

    [JsonPropertyName("taker_fee_rate")]
    public string TakerFeeRate { get; set; }

    [JsonPropertyName("last_filled_amount")]
    public string LastFilledAmount { get; set; }

    [JsonPropertyName("last_filled_price")]
    public string LastFilledPrice { get; set; }

    [JsonPropertyName("created_at")]
    public long CreatedAt { get; set; }

    [JsonPropertyName("updated_at")]
    public long UpdatedAt { get; set; }

    [JsonPropertyName("status")]
    public string Status { get; set; }
}


class CoinExHttpClient : IDisposable
{
    private string baseUrl;
    private string key;
    private string secret;
    private HttpClient client;

    public CoinExHttpClient(string baseUrl, string key, string secret)
    {
        this.baseUrl = baseUrl;
        this.key = key;
        this.secret = secret;
        client = new();
    }

    private string Sign(string method, string path, string body, long timestamp)
    {
        var message = method + path + body + timestamp.ToString();
        using (var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(secret)))
        {
            var r = hmac.ComputeHash(Encoding.UTF8.GetBytes(message));
            return BitConverter.ToString(r).Replace("-", "").ToLower();
        }
    }

    private async Task<T> Request<T>(string method, string path, Dictionary<string, object>? args, Dictionary<string, object>? body)
    {
        if (args != null)
        {
            var _args = args.Select(x => new KeyValuePair<string, string>(x.Key, x.Value.ToString()!));
            var query = await new FormUrlEncodedContent(_args).ReadAsStringAsync();
            path += "?" + query;
        }
        var req = new HttpRequestMessage(new(method), baseUrl + path);
        var bodyContent = "";
        if (body != null)
        {
            bodyContent = JsonSerializer.Serialize(body);
            req.Content = new StringContent(bodyContent, Encoding.UTF8, "application/json");
        }

        var now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        req.Headers.Add("X-COINEX-KEY", key);
        req.Headers.Add("X-COINEX-SIGN", Sign(method, req.RequestUri!.PathAndQuery, bodyContent, now));
        req.Headers.Add("X-COINEX-TIMESTAMP", now.ToString());

        var response = await client.SendAsync(req);
        response.EnsureSuccessStatusCode();
        var resp = await response.Content.ReadFromJsonAsync<CoinExHttpResp<T>>();
        resp!.EnsureSuccessStatusCode();
        return resp.Data;
    }

    public async Task<T> Get<T>(string path, Dictionary<string, object>? args = null) => await Request<T>("GET", path, args, null);

    public async Task<T> Post<T>(string path, Dictionary<string, object> body) => await Request<T>("POST", path, null, body);

    public async Task<Order> PutLimitOrder(string market, string side, string amount, string price)
    {
        return await Post<Order>("/spot/order", new() {
            { "market", market },
            { "market_type", "SPOT" },
            { "side", side },
            { "type", "limit" },
            { "amount", amount },
            { "price", price }
        });
    }

    public async Task<Order> QueryOrder(string market, long OrderId)
    {
        return await Get<Order>("/spot/order-status", new() {
            { "market", market },
            { "order_id", OrderId }
        });
    }

    public void Dispose() => client.Dispose();
}


class Program
{
    static async Task Main(string[] args)
    {
        using (var client = new CoinExHttpClient("https://api.coinex.com/v2", "YOUR_API_KEY", "YOUR_API_SECRET"))
        {
            var order = await client.PutLimitOrder("BTCUSDT", "buy", "0.01", "100000");
            Console.WriteLine(order.OrderId);

            await Task.Delay(10000);

            order = await client.QueryOrder("BTCUSDT", order.OrderId);
            Console.WriteLine(order.Status);
        }
    }
}