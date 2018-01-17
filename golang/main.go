package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"time"
)

var (
	ErrCoinexWrongMehtod = errors.New("request coinex api with wrong request mehtod")
	ErrCoinexWrongStatus = errors.New("request coinex api with wrong request status")
)

type coinexHTTPClient struct {
	baseURL *url.URL

	coinexKey    string
	coinexSecret string

	cli *http.Client
}

func NewCoinexHTTPClient(
	baseURL,

	coinexKey string,
	coinexSecret string,
) (*coinexHTTPClient, error) {
	base, err := url.Parse(baseURL)
	if err != nil {
		return nil, err
	}

	return &coinexHTTPClient{
		baseURL:      base,
		coinexKey:    coinexKey,
		coinexSecret: coinexSecret,
		cli:          http.DefaultClient,
	}, nil
}

type coinexResp struct {
	Code    int             `json:"code"`
	Data    json.RawMessage `json:"data"`
	Message string          `json:"message"`
}

func (c *coinexHTTPClient) request(method, path string, params url.Values, body map[string]interface{}) ([]byte, error) {
	if method == http.MethodGet {
		path = path + "?" + params.Encode()
	} else if method == http.MethodPost {
	} else {
		return nil, ErrCoinexWrongMehtod
	}

	var (
		err       error
		bodyBytes []byte
	)
	if body != nil {
		bodyBytes, err = json.Marshal(body)
		if err != nil {
			return nil, err
		}
	}
	// build request
	req, err := http.NewRequest(method, c.baseURL.String()+path, bytes.NewBuffer(bodyBytes))
	if err != nil {
		return nil, err
	}

	now := time.Now().UnixMilli()

	// set header
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-COINEX-KEY", c.coinexKey)
	req.Header.Set("X-COINEX-SIGN", c.sign(method, c.baseURL.Path+path, string(bodyBytes), int(now)))
	req.Header.Set("X-COINEX-TIMESTAMP", strconv.Itoa(int(now)))

	resp, err := c.cli.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	bodyBytes, err = io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	// check status code first
	if resp.StatusCode != http.StatusOK {
		fmt.Printf("request coinex api with wrong status: %d and body: %s", resp.StatusCode, string(bodyBytes))
		return nil, ErrCoinexWrongStatus
	}

	// retrieve data
	var r coinexResp
	err = json.Unmarshal(bodyBytes, &r)
	if err != nil {
		return nil, err
	}

	if r.Code != 0 {
		fmt.Printf("request coinex api with wrong biz code: %d and body: %s", r.Code, string(bodyBytes))
		return nil, errors.New(r.Message)
	}
	return r.Data, nil
}

func (c *coinexHTTPClient) sign(method, path string, body string, timestamp int) string {
	preparedStr := method + path + body + strconv.Itoa(timestamp) + c.coinexSecret
	hash := sha256.Sum256([]byte(preparedStr))
	return strings.ToLower(hex.EncodeToString(hash[:]))
}

func (c *coinexHTTPClient) GET(path string, params url.Values) ([]byte, error) {
	return c.request(http.MethodGet, path, params, nil)
}

func (c *coinexHTTPClient) POST(path string, body map[string]interface{}) ([]byte, error) {
	return c.request(http.MethodPost, path, nil, body)
}

type AccountInfo struct {
	SubUserName  string   `json:"sub_user_name"`
	IsFrozen     bool     `json:"is_frozen"`
	IsAuthorized bool     `json:"is_authorized"`
	Permissions  []string `json:"permissions"`
	BalanceUsd   string   `json:"balance_usd"`
}

type AccountList []*AccountInfo

func (c *coinexHTTPClient) GetAccountList(subUserName string, isFrozen bool) (AccountList, error) {
	params := url.Values{}
	params.Add("sub_user_name", subUserName)
	params.Add("is_frozen", strconv.FormatBool(isFrozen))
	resp, err := c.GET("/account/subs", params)
	if err != nil {
		return nil, err
	}
	var accountList AccountList
	err = json.Unmarshal(resp, &accountList)
	if err != nil {
		return nil, err
	}
	return accountList, err
}

type SpotOrderResp struct {
	Amount         string `json:"amount"`
	BaseFee        string `json:"base_fee"`
	Ccy            string `json:"ccy"`
	ClientID       string `json:"client_id"`
	CreatedAt      int64  `json:"created_at"`
	DiscountFee    string `json:"discount_fee"`
	FilledAmount   string `json:"filled_amount"`
	LastFillAmount string `json:"last_fill_amount"`
	LastFillPrice  string `json:"last_fill_price"`
	MakerFeeRate   string `json:"maker_fee_rate"`
	Market         string `json:"market"`
	MarketType     string `json:"market_type"`
	OrderID        int    `json:"order_id"`
	Price          string `json:"price"`
	QuoteFee       string `json:"quote_fee"`
	Side           string `json:"side"`
	TakerFeeRate   string `json:"taker_fee_rate"`
	Type           string `json:"type"`
	UnfilledAmount string `json:"unfilled_amount"`
	UpdatedAt      int64  `json:"updated_at"`
}

func (c *coinexHTTPClient) PutLimitOrder(market, side, orderType, amount, price string) (*SpotOrderResp, error) {
	resp, err := c.POST("/spot/order", map[string]interface{}{
		"market":      market,
		"market_type": "spot",
		"side":        side,
		"type":        orderType,
		"amount":      amount,
		"price":       price,
	})
	if err != nil {
		return nil, err
	}
	var orderResp SpotOrderResp
	err = json.Unmarshal(resp, &orderResp)
	if err != nil {
		return nil, err
	}
	return &orderResp, err
}

func (c *coinexHTTPClient) PutMarketOrder(market, side, amount string) (*SpotOrderResp, error) {
	resp, err := c.POST("/spot/order", map[string]interface{}{
		"market":      market,
		"market_type": "spot",
		"side":        side,
		"type":        "market",
		"amount":      amount,
	})
	if err != nil {
		return nil, err
	}
	var orderResp SpotOrderResp
	err = json.Unmarshal(resp, &orderResp)
	if err != nil {
		return nil, err
	}
	return &orderResp, err
}

type ListSpotOrdersResp []*SpotOrderResp

func (c *coinexHTTPClient) QueryPendingOrder(market, market_type, side string, page, limit int) (ListSpotOrdersResp, error) {
	params := url.Values{}
	params.Add("market", market)
	params.Add("market_type", market_type)
	params.Add("side", side)
	params.Add("page", strconv.Itoa(page))
	params.Add("limit", strconv.Itoa(limit))

	resp, err := c.GET("/spot/pending-order", params)
	if err != nil {
		return nil, err
	}

	var listOrderResp ListSpotOrdersResp
	err = json.Unmarshal(resp, &listOrderResp)
	if err != nil {
		return nil, err
	}
	return listOrderResp, err
}

// QueryOrderFinished Acquire executed order list
func (c *coinexHTTPClient) QueryFinishedOrder(market, market_type, side string, page, limit int) (ListSpotOrdersResp, error) {
	params := url.Values{}
	params.Add("market", market)
	params.Add("market_type", market_type)
	params.Add("side", side)
	params.Add("page", strconv.Itoa(page))
	params.Add("limit", strconv.Itoa(limit))

	resp, err := c.GET("/spot/finished-order", params)
	if err != nil {
		return nil, err
	}

	var listOrderResp ListSpotOrdersResp
	err = json.Unmarshal(resp, &listOrderResp)
	if err != nil {
		return nil, err
	}
	return listOrderResp, err
}

func main() {
	// get user sub account list
	client, err := NewCoinexHTTPClient(
		"https://api.coinex.com/v2",
		"xxxxxx",
		"yyyyyy",
	)
	if err != nil {
		panic(err)
	}

	data, err := client.QueryPendingOrder(
		"",
		"spot",
		"buy",
		1,
		10,
	)
	if err != nil {
		panic(err)
	}
	// list resp
	for i := 0; i < len(data); i++ {
		fmt.Printf("%+v\n", data[i])
	}
}
