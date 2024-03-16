# -*- coding: utf-8 -*-
import hashlib
import json
import time
from urllib.parse import urlparse

import requests

access_id = ""  # Replace with your access id
secret_key = ""  # Replace with your secret key


class RequestsClient(object):
    HEADERS = {
        "Content-Type": "application/json; charset=utf-8",
        "Accept": "application/json",
        "X-COINEX-KEY": "",
        "X-COINEX-SIGN": "",
        "X-COINEX-TIMESTAMP": "",
    }

    def __init__(self):
        self.access_id = access_id
        self.secret_key = secret_key
        self.url = "https://api.coinex.com/v2"
        self.headers = self.HEADERS.copy()

    # Generate your signature string
    def gen_sign(self, method, request_path, body, timestamp):
        prepared_str = f"{method}{request_path}{body}{timestamp}{self.secret_key}"
        signed_str = hashlib.sha256(prepared_str.encode("utf-8")).hexdigest().lower()
        return signed_str

    def get_common_headers(self, signed_str, timestamp):
        headers = self.HEADERS.copy()
        headers["X-COINEX-KEY"] = self.access_id
        headers["X-COINEX-SIGN"] = signed_str
        headers["X-COINEX-TIMESTAMP"] = timestamp
        headers["Content-Type"] = "application/json; charset=utf-8"
        return headers

    def request(self, method, url, params={}, data=""):
        req = urlparse(url)
        request_path = req.path

        timestamp = str(int(time.time() * 1000))
        if method.upper() == "GET":
            # If params exist, query string needs to be added to the request path
            if params:
                query_params = []
                for item in params:
                    if params[item] is None:
                        continue
                    query_params.append(item + "=" + str(params[item]))
                query_string = "?{0}".format("&".join(query_params))
                request_path = request_path + query_string

            signed_str = self.gen_sign(
                method, request_path, body="", timestamp=timestamp
            )
            response = requests.get(
                url,
                params=params,
                headers=self.get_common_headers(signed_str, timestamp),
            )

        else:
            signed_str = self.gen_sign(
                method, request_path, body=data, timestamp=timestamp
            )
            response = requests.post(
                url, data, headers=self.get_common_headers(signed_str, timestamp)
            )

        if response.status_code != 200:
            raise ValueError(response.text)
        return response


request_client = RequestsClient()


def get_spot_market():
    request_path = "/spot/market"
    params = {"market": "BTCUSDT"}
    response = request_client.request(
        "GET",
        "{url}{request_path}".format(url=request_client.url, request_path=request_path),
        params=params,
    )
    return response


def get_spot_balance():
    request_path = "/assets/spot/balance"
    response = request_client.request(
        "GET",
        "{url}{request_path}".format(url=request_client.url, request_path=request_path),
    )
    return response


def get_deposit_address():
    request_path = "/assets/deposit-address"
    params = {"ccy": "USDT", "chain": "CSC"}

    response = request_client.request(
        "GET",
        "{url}{request_path}".format(url=request_client.url, request_path=request_path),
        params=params,
    )
    return response


def put_limit():
    request_path = "/spot/order"
    data = {
        "market": "BTCUSDT",
        "market_type": "SPOT",
        "side": "buy",
        "type": "limit",
        "amount": "10000",
        "price": "1",
        "client_id": "user1",
        "is_hide": True,
    }
    data = json.dumps(data)
    response = request_client.request(
        "POST",
        "{url}{request_path}".format(url=request_client.url, request_path=request_path),
        data=data,
    )
    return response


def run_code():
    try:
        response_1 = get_spot_market().json()
        print(response_1)

        response_2 = get_spot_balance().json()
        print(response_2)

        response_3 = get_deposit_address().json()
        print(response_3)

        response_4 = put_limit().json()
        print(response_4)

    except Exception as e:
        print("Error:" + str(e))
        time.sleep(3)
        run_code()


if __name__ == "__main__":
    run_code()
