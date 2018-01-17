const ACCESS_ID = ""; // your access id
const SECRET_KEY = ""; // your secret key


const crypto = require("crypto");
function createAuthorization(method, request_path, body_json, timestamp) {
    var text = method + request_path + body_json + timestamp + SECRET_KEY;
    console.log(text);
    return crypto
        .createHash("sha256")
        .update(text)
        .digest("hex")
        .toUpperCase();
}

const Axios = require("axios");
const axios = Axios.create({
    baseURL: "https://api.coinex.com/",
    headers: {
        "User-Agent":
            "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/39.0.2171.71 Safari/537.36",
        post: {
            "Content-Type": "application/json",
        },
    },
    timeout: 10000,
});


/**
 *  demo
 */

async function getSpotMaketList() {
    const res = await axios.get("/v2/spot/market");
    console.log("market list:\n", JSON.stringify(res.data, null, 2));
}

async function getSpotBalance() {
    timetamp = Date.now();
    const res = await axios.get("/v2/assets/spot/balance", {
        headers: {
            "X-COINEX-KEY": ACCESS_ID,
            "X-COINEX-SIGN": createAuthorization("GET", "/v2/assets/spot/balance", "", timetamp),
            "X-COINEX-TIMESTAMP": timetamp,
        }
    });
    console.log("account info:\n", JSON.stringify(res.data, null, 2));
}

async function putLimitOrder() {
    timetamp = Date.now();
    const data = JSON.stringify({
        market: "BTCUSDT",
        market_type: "SPOT",
        side: "buy",
        type: "limit",
        amount: "0.001",
        price: "30000",
    });
    const res = await axios.post("/v2/spot/order", data, {
        headers: {
            "X-COINEX-KEY": ACCESS_ID,
            "X-COINEX-SIGN": createAuthorization("POST", "/v2/spot/order", data, timetamp),
            "X-COINEX-TIMESTAMP": timetamp,
        }
    });
    console.log("place limit order:\n", JSON.stringify(res.data, null, 2));
}

async function putMarketOrder() {
    timetamp = Date.now();
    const data = JSON.stringify({
        market: "BTCUSDT",
        market_type: "SPOT",
        side: "buy",
        type: "market",
        amount: "0.50",
        price: "42500",
    });
    const res = await axios.post("/v2/spot/order", data, {
        headers: {
            "X-COINEX-KEY": ACCESS_ID,
            "X-COINEX-SIGN": createAuthorization("POST", "/v2/spot/order", data, timetamp),
            "X-COINEX-TIMESTAMP": timetamp,
        }
    });
    console.log("place market order:\n", JSON.stringify(res.data, null, 2));
}

async function listPendingOrder() {
    timetamp = Date.now();
    const data = {
        market: "BTCUSDT",
        market_type: "SPOT",
        side: "buy",
        client_id: "",
        page: 1,
        limit: 10,
    };
    queryString = Object.keys(data)
        .map(key => `${encodeURIComponent(key)}=${encodeURIComponent(data[key])}`)
        .join('&');
    requestPath = "/v2/spot/pending-order" + "?" + queryString;

    const res = await axios.get(requestPath, {
        headers: {
            "X-COINEX-KEY": ACCESS_ID,
            "X-COINEX-SIGN": createAuthorization("GET", requestPath, "", timetamp),
            "X-COINEX-TIMESTAMP": timetamp,
        }
    });
    const pendingOrders = res.data.code === 0 ? res.data.data : [];
    console.log("pending orders:\n", JSON.stringify(res.data, null, 2));
    return pendingOrders;
}


async function cancelOrder(market, id) {
    timetamp = Date.now();
    const data = JSON.stringify({
        market: market,
        market_type: "SPOT",
        order_id: id,
    });
    const res = await axios.post("/v2/spot/cancel-order", data, {
        headers: {
            "X-COINEX-KEY": ACCESS_ID,
            "X-COINEX-SIGN": createAuthorization("POST", "/v2/spot/cancel-order", data, timetamp),
            "X-COINEX-TIMESTAMP": timetamp,
        }
    });
    console.log("cancel order:\n", JSON.stringify(res.data, null, 2));
}


async function demo() {
    await getSpotMaketList();

    await getSpotBalance();

    await putLimitOrder();

    await putMarketOrder();

    const orders = await listPendingOrder();
    orders.forEach(function (order) {
        cancelOrder(order.market, order.order_id);
    });
}
demo();

















