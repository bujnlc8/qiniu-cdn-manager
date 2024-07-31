use std::path::PathBuf;
use std::{io, str::FromStr};

use chrono::Local;
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use qiniu_cdn_manager::{
    analysis::top::FilterType,
    config::Config,
    utils::{get_domains, print_err, prompt, qy_robot::QyRobot, wait_blink},
    Client, SubFunctionEnum, QUERYING,
};

use colored::Colorize;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 配置文件路径, 如果未传, 从当前目录找配置文件, 如果不存在, 尝试加载`$HOME/.config/qiniu-cdn.toml`
    #[arg(short, long)]
    config: Option<String>,

    /// CDN域名, 会覆盖配置文件的domain字段
    #[arg(short, long)]
    domain: Option<String>,

    /// 开启debug模式
    #[clap(long, action)]
    debug: bool,

    /// 生成shell补全脚本, 支持Bash, Zsh, Fish, PowerShell, Elvish
    #[arg(long)]
    completion: Option<String>,

    /// Do not print time(ms) elapsed
    #[clap(short, long)]
    no_elapsed: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 查询请求次数
    Count(CountArgs),

    /// 诊断疑似IP
    Diagnostic(DiagnosticArgs),

    /// 查询命中率
    Hitmiss(HitissArgs),

    /// 查询域名信息
    Info(InfoArgs),

    /// IP黑/白名单
    Ipacl(IPaclArgs),

    /// 查询IP的URL请求次数
    Ipurl(IPUrlArgs),

    /// 查询运营商请求次数
    ISPCount(ISPCountArgs),

    /// 查询运营商流量
    ISPTraffic(ISPTrafficArgs),

    /// 查询运营商流量占比
    ISPTrafficRatio(ISPTrafficRatioArgs),

    /// 下载请求日志
    LogDownload(LogDownloadArgs),

    /// 过滤请求日志
    LogFilter(LogFilterArgs),

    /// 文件预取
    Prefetch(PrefetchArgs),

    /// 刷新CDN缓存
    Refresh(RefreshArgs),

    /// 查询状态码
    Status(StatusArgs),

    /// 查询TOP请求
    Top(TopArgs),

    /// 查询计费流量
    Traffic(TrafficArgs),
}

#[derive(Args)]
struct LogDownloadArgs {
    /// 日期, 例如 2016-07-01, 默认当天
    #[arg(short, long)]
    day: Option<String>,

    /// 下载条数, 默认全部
    #[arg(short, long)]
    limit: Option<i32>,

    /// 下载目录, 默认当前目录的`./logs`目录
    #[arg(long)]
    dir: Option<String>,

    /// 不要将日志放在域名目录里, 和在配置文件设置`download_log_domain_dir=false`同义
    #[clap(long, short, action)]
    no_domain_dir: bool,

    /// 解压缩日志并保留源压缩文件
    #[clap(long, action, conflicts_with = "unzip_not_keep")]
    unzip_keep: bool,

    /// 解压缩日志不保留源压缩文件
    #[clap(long, action, conflicts_with = "unzip_keep")]
    unzip_not_keep: bool,
}

#[derive(Args)]
struct RefreshArgs {
    /// 要刷新的url列表, 总数不超过60条, 多条以英文逗号隔开, 如：http://bar.foo.com/index.html
    #[arg(short, long)]
    urls: Option<String>,

    /// 要刷新的目录url列表, 总数不超过10条, 多条以英文逗号隔开, 如: http://bar.foo.com/dir/
    #[arg(short, long)]
    dirs: Option<String>,
}

#[derive(Args)]
struct PrefetchArgs {
    /// 要预取url列表, 总数不超过60条, 多条以英文逗号隔开, 如：http://bar.foo.com/test.zip
    #[arg(short, long)]
    urls: String,
}

#[derive(Args)]
struct TrafficArgs {
    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// 粒度, 取值：5min/hour/day,  默认day
    #[arg(short, long)]
    granularity: Option<String>,

    /// 不要打印流量明细
    #[clap(long, action)]
    no_print: bool,

    /// 不要发送流量告警
    #[clap(long, action)]
    no_warn: bool,

    /// 每5分钟流量告警(MB)阈值
    #[arg(long)]
    five_minute_traffic: Option<i64>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,
}

#[derive(Args)]
struct TopArgs {
    /// 区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global
    #[arg(short, long)]
    region: Option<String>,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// IP模式(默认)
    #[clap(long, short, action, conflicts_with = "url")]
    ip: bool,

    /// URL模式
    #[clap(long, short, action, conflicts_with = "ip")]
    url: bool,

    /// 根据流量查询(默认)
    #[clap(long, short, action, conflicts_with = "count")]
    traffic: bool,

    /// 根据请求数量查询
    #[clap(long, short, action, conflicts_with = "traffic")]
    count: bool,

    /// 输出条数, 默认全部
    #[arg(short, long)]
    limit: Option<i32>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,
}

#[derive(Args)]
struct StatusArgs {
    /// 区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global, 多个用英文逗号隔开
    #[arg(short, long)]
    regions: Option<String>,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// 输出条数, 默认全部
    #[arg(short, long)]
    limit: Option<i32>,

    /// ISP运营商, 比如all(所有 ISP), telecom(电信), unicom(联通), mobile(中国移动), drpeng(鹏博士), tietong(铁通), cernet(教育网)
    #[arg(short, long)]
    isp: Option<String>,

    /// 粒度, 可选项为 5min、1hour、1day, 默认1day
    #[arg(short, long)]
    freq: Option<String>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,
}

#[derive(Args)]
struct InfoArgs {
    /// 下载ssl证书
    #[clap(long, short, action, conflicts_with = "list_all_domain")]
    download_ssl_cert: bool,

    /// 列出账户下绑定的所有域名
    #[clap(long, short, action, conflicts_with = "download_ssl_cert")]
    list_all_domain: bool,
}

#[derive(Args)]
struct CountArgs {
    /// 粒度, 可选项为 5min、1hour、1day, 默认1day
    #[arg(short, long)]
    freq: Option<String>,

    /// 区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global
    #[arg(short, long)]
    region: Option<String>,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// 输出条数, 默认全部
    #[arg(short, long)]
    limit: Option<i32>,

    /// 不要发送次数告警
    #[clap(long, short, action)]
    no_warn: bool,

    /// 每5分钟请求次数告警阈值
    #[arg(long)]
    five_minute_count: Option<i64>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,
}

#[derive(Args)]
struct HitissArgs {
    /// 粒度, 可选项为 5min、1hour、1day, 默认1day
    #[arg(short, long)]
    freq: Option<String>,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// 输出条数, 默认全部
    #[arg(short, long)]
    limit: Option<i32>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,
}

#[derive(Args)]
struct ISPTrafficArgs {
    /// 粒度, 可选项为 5min、1hour、1day, 默认1day
    #[arg(short, long)]
    freq: Option<String>,

    /// 区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global, 多个以英文逗号隔开
    #[arg(short, long)]
    regions: Option<String>,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// ISP运营商, 比如all(所有 ISP), telecom(电信), unicom(联通), mobile(中国移动), drpeng(鹏博士), tietong(铁通), cernet(教育网)
    #[arg(short, long)]
    isp: Option<String>,

    /// 输出条数, 默认全部
    #[arg(short, long)]
    limit: Option<i32>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,

    /// 按区域排序，会查询所有区域并按流量从大到小排序
    #[clap(long, action, conflicts_with_all = vec!["regions", "isp_sort"])]
    region_sort: bool,

    /// 按运营商排序，会查询所有运营商并按流量从大到小排序
    #[clap(long, action, conflicts_with_all = vec!["isp", "region_sort"])]
    isp_sort: bool,
}

#[derive(Args)]
struct ISPCountArgs {
    /// 粒度, 可选项为 5min、1hour、1day, 默认1day
    #[arg(short, long)]
    freq: Option<String>,

    /// 区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global
    #[arg(short, long)]
    region: Option<String>,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// 输出条数, 默认全部
    #[arg(short, long)]
    limit: Option<i32>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,

    /// 按区域排序，会查询所有区域并按请求次数从大到小排序
    #[clap(long, action, conflicts_with = "region")]
    region_sort: bool,
}

#[derive(Args)]
struct ISPTrafficRatioArgs {
    /// 区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global, 多个以英文逗号隔开
    #[arg(short, long)]
    regions: Option<String>,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,
}

#[derive(Args)]
struct IPaclArgs {
    /// 设置白名单
    #[clap(long, short, action, conflicts_with_all = vec!["black", "close"])]
    white: bool,

    /// 设置黑名单
    #[clap(long, short, action, conflicts_with_all = vec!["white", "close"])]
    black: bool,

    /// 关闭黑白名单
    #[clap(long, short, action, conflicts_with_all = vec!["white", "black"])]
    close: bool,

    /// 覆盖已存在的ip列表，默认是追加模式
    #[clap(long, short, action)]
    rewrite: bool,

    /// ip列表, 多个以英文逗号隔开, 以`d`开头表示移除(模式需保证为no-rewrite)
    #[arg(short, long)]
    ips: Option<String>,

    /// 不发送企业微信通知
    #[clap(long, action)]
    no_qy_notify: bool,
}

#[derive(Args)]
struct DiagnosticArgs {
    /// 配置IP黑名单
    #[clap(long, short, action)]
    apply_black_ip: bool,

    /// 不覆盖已存在的ip列表，采用追加模式
    #[clap(long, short, action)]
    no_rewrite: bool,

    /// 不发送企业微信通知
    #[clap(long, action)]
    no_qy_notify: bool,

    /// 不弹出prompt确认
    #[clap(long, action)]
    no_prompt: bool,

    /// 诊断策略，支持T:1:200和C:1:10000这两种格式, 可以用&&和||连接(并用引号引起来)
    #[arg(short, long)]
    policy: Option<String>,

    /// 包含所有域名
    #[clap(long, action, conflicts_with = "domains")]
    all_domain: bool,

    /// 排除域名，多个以英文逗号隔开
    #[arg(long)]
    domain_exclude: Option<String>,

    /// 域名，多个以英文逗号隔开
    #[arg(long, conflicts_with = "all_domain")]
    domains: Option<String>,

    /// 诊断结束日, 默认当天, 如2016-07-01
    #[arg(long)]
    day: Option<String>,
}

#[derive(Args)]
struct IPUrlArgs {
    /// 要查询的IP, 请求日志一般滞后6个小时左右
    #[arg(short, long)]
    ip: String,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// 输出条数, 默认全部
    #[arg(short, long)]
    limit: Option<i32>,
}

#[derive(Args)]
struct LogFilterArgs {
    /// 要过滤的字符串，支持传递多次, 以!!开头表示不包含
    #[arg(short, long)]
    filter_string: Vec<String>,

    /// 开始日期, 例如：2016-07-01, 默认当天
    #[arg(short, long)]
    start_date: Option<String>,

    /// 结束日期, 例如：2016-07-03, 默认当天
    #[arg(short, long)]
    end_date: Option<String>,

    /// 输出到文件
    #[clap(long, action)]
    output_file: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let config_path = cli.config.map(PathBuf::from);
    let mut config = Config::parse(config_path);
    if let Some(domain) = cli.domain {
        config.cdn.domain = domain;
    }
    if cli.debug {
        config.debug = Some(true);
    }
    let today = Local::now().format("%Y-%m-%d").to_string();
    let start = Local::now().timestamp_millis();
    match &cli.command {
        Some(command) => match command {
            // 下载日志
            Commands::LogDownload(args) => {
                let day = args.day.clone().unwrap_or(today.clone());
                if args.no_domain_dir {
                    config.download_log_domain_dir = Some(false);
                }
                let client = Client::new(&config, SubFunctionEnum::Log);
                let dir = args.dir.clone().unwrap_or("./logs".to_string());
                client
                    .download(
                        &day,
                        Some(&dir),
                        args.limit,
                        args.unzip_keep,
                        args.unzip_not_keep,
                        &config.cdn.domain,
                    )
                    .await?;
            }
            // 刷新缓存
            Commands::Refresh(args) => {
                let urls = args.urls.clone().unwrap_or("".to_string());
                let dirs = args.dirs.clone().unwrap_or("".to_string());
                if urls.is_empty() && dirs.is_empty() {
                    print_err("待刷新的链接或目录为空！", true);
                }
                let client = Client::new(&config, SubFunctionEnum::Refresh);
                let response = client.refresh(&urls, &dirs).await?;
                client.print_result(&response);
            }
            // 计费流量查询
            Commands::Traffic(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let granularity = args.granularity.clone().unwrap_or("day".to_string());
                if let Some(five_minute_traffic) = args.five_minute_traffic {
                    config.five_minute_traffic = Some(five_minute_traffic);
                }
                let mut client = Client::new(&config, SubFunctionEnum::Traffic);
                // 所有域名的流量
                if args.all_domain || args.domains.is_some() {
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    let domains = get_domains(&config, &args.domain_exclude, &args.domains).await?;
                    client
                        .all_domain_charge_traffic(
                            &start_date,
                            &end_date,
                            &granularity,
                            args.no_warn,
                            domains,
                        )
                        .await?;
                } else {
                    let response = client
                        .charge_traffic(&start_date, &end_date, &granularity, &config.cdn.domain)
                        .await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    client
                        .print_traffic_result(
                            &response,
                            args.no_print,
                            &start_date,
                            &end_date,
                            &granularity,
                            args.no_warn,
                            &config.cdn.domain,
                        )
                        .await?;
                }
            }
            //域名信息
            Commands::Info(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let client = Client::new(&config, SubFunctionEnum::Domain);
                if args.list_all_domain {
                    let response = client.domain_list().await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    if response.domains.is_empty() {
                        println!("{}", "未找到绑定的域名".yellow().bold());
                    } else {
                        println!(
                            "{}",
                            format!("该账户下绑定的所有域名({}): ", response.domains.len())
                                .green()
                                .bold()
                        );
                        for d in response.domains {
                            println!("{} {}", d.name, d.operating_state.unwrap().dimmed());
                        }
                    }
                } else {
                    let response = client.domain_info(&config.cdn.domain).await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    client
                        .print_domain_info(&response, args.download_ssl_cert, &config.cdn.domain)
                        .await;
                }
            }
            // TOP查询
            Commands::Top(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let region = args.region.clone().unwrap_or("global".to_string());
                let mut mode = "ip";
                if args.url {
                    mode = "url";
                }
                let mut filter_type = FilterType::Traffic;
                if args.count {
                    filter_type = FilterType::ReqCount;
                }
                let client = Client::new(&config, SubFunctionEnum::AnalysisTop);
                let domains = if args.all_domain || args.domains.is_some() {
                    get_domains(&config, &args.domain_exclude, &args.domains).await?
                } else {
                    vec![config.cdn.domain.clone()]
                };
                if mode == "ip" {
                    let response = client
                        .top_ip(
                            &region,
                            &start_date,
                            &end_date,
                            filter_type,
                            domains.clone(),
                        )
                        .await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    client.print_top_ip(
                        response,
                        filter_type,
                        args.limit,
                        &start_date,
                        &end_date,
                        &region,
                        domains,
                    );
                } else {
                    let response = client
                        .top_url(
                            &region,
                            &start_date,
                            &end_date,
                            filter_type,
                            domains.clone(),
                        )
                        .await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    client.print_top_url(
                        response,
                        filter_type,
                        args.limit,
                        &start_date,
                        &end_date,
                        &region,
                        domains,
                    );
                }
            }
            // 状态码查询
            Commands::Status(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let regions = args.regions.clone().unwrap_or("global".to_string());
                let freq = args.freq.clone().unwrap_or("1day".to_string());
                let isp = args.isp.clone().unwrap_or("all".to_string());
                let client = Client::new(&config, SubFunctionEnum::AnalysisStatus);
                let domains = if args.all_domain || args.domains.is_some() {
                    get_domains(&config, &args.domain_exclude, &args.domains).await?
                } else {
                    vec![config.cdn.domain.clone()]
                };
                let response = client
                    .status_code(
                        freq.into(),
                        &regions,
                        &isp,
                        &start_date,
                        &end_date,
                        domains.clone(),
                    )
                    .await?;
                if let Some(blinker) = blinker {
                    blinker.sender.send(true).unwrap();
                    blinker.handle.await?;
                }
                client.print_status(
                    response,
                    args.limit,
                    &isp,
                    &regions,
                    &start_date,
                    &end_date,
                    domains,
                );
            }
            // 请求次数查询
            Commands::Count(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let region = args.region.clone().unwrap_or("global".to_string());
                let freq = args.freq.clone().unwrap_or("1day".to_string());
                if let Some(five_minute_count) = args.five_minute_count {
                    config.five_minute_count = Some(five_minute_count);
                }
                let mut client = Client::new(&config, SubFunctionEnum::AnalysisCount);
                if args.all_domain || args.domains.is_some() {
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    let domains = get_domains(&config, &args.domain_exclude, &args.domains).await?;
                    client
                        .all_domain_req_count(
                            freq.into(),
                            &region,
                            &start_date,
                            &end_date,
                            args.no_warn,
                            domains.clone(),
                        )
                        .await?;
                } else {
                    let domains = vec![config.cdn.domain.clone()];
                    let response = client
                        .req_count(
                            freq.clone().into(),
                            &region,
                            &start_date,
                            &end_date,
                            domains.clone(),
                        )
                        .await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    client
                        .print_count(
                            response,
                            args.limit,
                            &region,
                            &start_date,
                            &end_date,
                            freq.into(),
                            args.no_warn,
                            domains,
                        )
                        .await?;
                }
            }
            // 命中率查询
            Commands::Hitmiss(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let freq = args.freq.clone().unwrap_or("1day".to_string());
                let domains = if args.all_domain || args.domains.is_some() {
                    get_domains(&config, &args.domain_exclude, &args.domains).await?
                } else {
                    vec![config.cdn.domain.clone()]
                };
                let client = Client::new(&config, SubFunctionEnum::AnalysisHitmiss);
                let response = client
                    .hit_miss(freq.into(), &start_date, &end_date, domains.clone())
                    .await?;
                if let Some(blinker) = blinker {
                    blinker.sender.send(true).unwrap();
                    blinker.handle.await?;
                }
                client.print_hitmiss(response, args.limit, &start_date, &end_date, domains);
            }
            // 运营商流量查询
            Commands::ISPTraffic(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let regions = args.regions.clone().unwrap_or("global".to_string());
                let freq = args.freq.clone().unwrap_or("1day".to_string());
                let isp = args.isp.clone().unwrap_or("all".to_string());
                let domains = if args.all_domain || args.domains.is_some() {
                    get_domains(&config, &args.domain_exclude, &args.domains).await?
                } else {
                    vec![config.cdn.domain.clone()]
                };
                let client = Client::new(&config, SubFunctionEnum::AnalysisIsp);
                if args.region_sort || args.isp_sort {
                    client
                        .isp_traffic_sort(
                            freq.into(),
                            &regions,
                            &isp,
                            &start_date,
                            &end_date,
                            domains.clone(),
                            blinker,
                            args.isp_sort,
                        )
                        .await?;
                } else {
                    let response = client
                        .clone()
                        .isp_traffic(
                            freq.into(),
                            &regions,
                            &isp,
                            &start_date,
                            &end_date,
                            domains.clone(),
                        )
                        .await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    client.print_isp_traffic(
                        response,
                        args.limit,
                        &isp,
                        &regions,
                        &start_date,
                        &end_date,
                        domains,
                    );
                }
            }
            // ISP请求次数
            Commands::ISPCount(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let region = args.region.clone().unwrap_or("global".to_string());
                let freq = args.freq.clone().unwrap_or("1day".to_string());
                let domains = if args.all_domain || args.domains.is_some() {
                    get_domains(&config, &args.domain_exclude, &args.domains).await?
                } else {
                    vec![config.cdn.domain.clone()]
                };
                let client = Client::new(&config, SubFunctionEnum::AnalysisIsp);
                if args.region_sort {
                    client
                        .isp_count_all_region(
                            freq.into(),
                            &start_date,
                            &end_date,
                            domains.clone(),
                            blinker,
                        )
                        .await?;
                } else {
                    let response = client
                        .isp_count(
                            freq.into(),
                            &region,
                            &start_date,
                            &end_date,
                            domains.clone(),
                        )
                        .await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    client.print_isp_count(
                        response,
                        args.limit,
                        &region,
                        &start_date,
                        &end_date,
                        domains,
                    );
                }
            }
            // ISP流量占比
            Commands::ISPTrafficRatio(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let regions = args.regions.clone().unwrap_or("global".to_string());
                let client = Client::new(&config, SubFunctionEnum::AnalysisIsp);
                let domains = if args.all_domain || args.domains.is_some() {
                    get_domains(&config, &args.domain_exclude, &args.domains).await?
                } else {
                    vec![config.cdn.domain.clone()]
                };
                let response = client
                    .isp_traffic_ratio(&regions, &start_date, &end_date, domains.clone())
                    .await?;
                if let Some(blinker) = blinker {
                    blinker.sender.send(true).unwrap();
                    blinker.handle.await?;
                }
                client.print_traffic_ratio(response, &regions, &start_date, &end_date, domains);
            }
            // IP白名单/黑名单
            Commands::Ipacl(args) => {
                let client = Client::new(&config, SubFunctionEnum::Domain);
                let ips = args.ips.clone().unwrap_or("".to_string());
                if ips.contains('d') && args.rewrite {
                    print_err("存在需移除的IP, 模式需为no-rewrite！", true);
                }
                if args.black && !prompt("开启黑名单?", None) {
                    return Ok(());
                }
                if args.white && !prompt("开启白名单?", None) {
                    return Ok(());
                }
                if args.close && !prompt("关闭黑白名单?", None) {
                    return Ok(());
                }
                if !ips.is_empty() && !prompt(format!("IP: {}?", ips), None) {
                    return Ok(());
                }
                if args.rewrite {
                    if !prompt("覆盖已存在的配置?", None) {
                        return Ok(());
                    }
                } else if !prompt("追加到已存在的配置?", None) {
                    return Ok(());
                }
                client
                    .set_ip_acl(
                        args.black,
                        args.white,
                        args.close,
                        &ips,
                        args.rewrite,
                        &config.cdn.domain,
                    )
                    .await?;
                // 不发送企业微信消息
                if args.no_qy_notify || config.monitor.qy_robot.is_none() {
                    return Ok(());
                }
                let mut mode = "覆盖";
                if !args.rewrite {
                    mode = "追加";
                }
                let mut msg = "".to_string();
                if args.black || args.white {
                    let mut bw = "白";
                    if args.black {
                        bw = "黑";
                    }
                    msg = format!(
                            "## 🔔七牛云CDN IP黑/白名单修改\n\n`{}`采用`{}`模式添加了以下IP到{}名单:\n\n- {}\n\n> `d`开头表示移除\n\n🚀🚀🚀",
                            config.cdn.domain,
                            mode,
                            bw,
                            ips.replace(",", "\n\n- ")
                        );
                } else if args.close {
                    msg = format!(
                        "## 🔔七牛云CDN IP黑/白名单修改\n\n`{}`关闭IP黑/白名单\n\n🚀🚀🚀",
                        config.cdn.domain,
                    );
                }
                if !msg.is_empty() {
                    QyRobot::new(config.monitor.qy_robot.unwrap())
                        .send_message(&msg)
                        .await?;
                }
            }
            // 诊断疑似IP
            Commands::Diagnostic(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink("诊断中...".into(), 3))
                } else {
                    None
                };
                if args.no_rewrite {
                    config.blackip.rewrite = Some(false);
                }
                if args.policy.is_some() {
                    config.blackip.policy = args.policy.clone();
                }
                let mut client = Client::new(&config, SubFunctionEnum::Domain);
                let day = args.day.clone().unwrap_or(today);
                if args.all_domain || args.domains.is_some() {
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    let domains = get_domains(&config, &args.domain_exclude, &args.domains).await?;
                    client
                        .all_domain_diagnostic(
                            args.apply_black_ip,
                            args.no_prompt,
                            args.no_qy_notify,
                            domains,
                            &day,
                        )
                        .await?;
                } else {
                    let ips = client.diagnose_ip(&config.cdn.domain, &day).await?;
                    if let Some(blinker) = blinker {
                        blinker.sender.send(true).unwrap();
                        blinker.handle.await?;
                    }
                    client
                        .process_diagnostic_ips(
                            ips,
                            args.apply_black_ip,
                            args.no_prompt,
                            args.no_qy_notify,
                            &config.cdn.domain,
                        )
                        .await?;
                }
            }
            // IP请求URL的次数
            Commands::Ipurl(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let client = Client::new(&config, SubFunctionEnum::Log);
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                client
                    .ip_url(
                        &args.ip,
                        &start_date,
                        &end_date,
                        args.limit,
                        blinker,
                        &config.cdn.domain,
                    )
                    .await?;
            }
            // 文件预取
            Commands::Prefetch(args) => {
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink("操作中，请稍候...".to_string(), 3))
                } else {
                    None
                };
                let client = Client::new(&config, SubFunctionEnum::Prefetch);
                if let Some(blinker) = blinker {
                    blinker.sender.send(true).unwrap();
                    blinker.handle.await?;
                }
                let response = client.prefetch(&args.urls).await?;
                client.print_prefetch(response);
            }
            // 日志过滤
            Commands::LogFilter(args) => {
                if args.filter_string.is_empty() {
                    print_err("请输入待过滤的字符串！", true);
                }
                let blinker = if !config.debug.unwrap_or(false) {
                    Some(wait_blink(QUERYING.into(), 3))
                } else {
                    None
                };
                let start_date = args.start_date.clone().unwrap_or(today.clone());
                let end_date = args.end_date.clone().unwrap_or(today.clone());
                let client = Client::new(&config, SubFunctionEnum::Log);
                client
                    .filter_log(
                        args.filter_string.clone(),
                        &start_date,
                        &end_date,
                        args.output_file,
                        blinker,
                        &config.cdn.domain,
                    )
                    .await?;
            }
        },
        None => {
            // 生成shell补全脚本
            if let Some(shell) = cli.completion {
                let mut cmd = Cli::command();
                let bin_name = cmd.get_name().to_string();
                match Shell::from_str(&shell.to_lowercase()) {
                    Ok(shell) => {
                        generate(shell, &mut cmd, bin_name, &mut io::stdout());
                    }
                    Err(e) => print_err(e.to_string().as_str(), true),
                };
            } else {
                // 什么参数和命令都没有
                Cli::command().print_help()?;
            }
        }
    };
    if !cli.no_elapsed && cli.command.is_some() {
        println!(
            "{}{}ms",
            "Time Elapsed: ".cyan().bold(),
            Local::now().timestamp_millis() - start
        );
    }
    Ok(())
}
