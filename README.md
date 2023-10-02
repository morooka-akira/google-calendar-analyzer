# [calendar-schedule-analyzer](https://github.com/morooka-akira/calendar-schedule-analyzer)

Google カレンダーの空き時間を出力する。

## Setup

### 1.GoogleCalendarAPI を有効にする

https://console.cloud.google.com/apis/library/calendar-json.googleapis.com

### 2.OAuth 同意画面から認証キーをダウンロードする

【参考 URL】

- https://tech-broccoli.life/articles/engineer/use-google-calenadar-api-with-oauth#1-2-google-calendar-api%E3%82%92%E6%9C%89%E5%8A%B9%E3%81%AB%E3%81%99%E3%82%8B

作成したクライアントの認証情報の json をダウンロードする

### 3.環境変数にクレデンシャルの json ファイルの Path をセットする

```bash
export CALENDAR_CREDENTIAL=/path/to/credential.json
```

## Usage

- [Release](https://github.com/morooka-akira/google-calendar-analyzer/releases)から実行ファイルをダウンロード&解答する

```
chmod +x ./google-calendar-analyzer
./google-calendar-analyzer
```

or

```bash
cargo run
```

## Config

config.yaml ファイルに設定を入力し、実行ファイルと同ディレクトリ(プロジェクトの場合はプロジェクトルート)に配置してください。

```yaml
calendar_id: example@gmail.com
start_date: 2023-09-29
end_date: 2023-10-01
start_time: 09:00
end_time: 18:00
day_of_weeks: "0,1,2,3,4"
```

| 項目名      | 説明                                                                                        |
| ----------- | ------------------------------------------------------------------------------------------- |
| calendar_id | 検索するカレンダーの ID。Google カレンダーの場合は、カレンダーの設定画面から確認できる ID。 |
| start_date  | 検索開始日付。ISO 8601 形式で指定する。                                                     |
| end_date    | 検索終了日付。ISO 8601 形式で指定する。                                                     |
| start_time  | 検索開始時間。'hh:mm'形式で指定する。                                                       |
| end_time    | 検索終了時間。'hh:mm'形式で指定する。                                                       |
| day_of_week | 曜日を指定する。0（月）から 6（日）までの数値で指定する。                                   |
