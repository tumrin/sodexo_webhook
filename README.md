# sodexo_webhook

A Webhook for getting a lunch menu from a Sodexo restaurant and posting it on Discord daily.

## Setup

Create a .env file or set environment variables

```
sodexo_url={} # Get this from sodexo restaurant's webpage eg. "https://www.sodexo.fi/ruokalistat/output/daily_json/98/"
webhook_url={} # Your Discord webhook url
interval_time={} # Interval time as hh:mm eg. "07:00"
```

Run the webhook

```
cargo run --release
```
