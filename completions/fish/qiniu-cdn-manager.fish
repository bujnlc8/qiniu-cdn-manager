# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_qiniu_cdn_manager_global_optspecs
	string join \n c/config= d/domain= debug completion= n/no-elapsed h/help V/version
end

function __fish_qiniu_cdn_manager_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_qiniu_cdn_manager_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_qiniu_cdn_manager_using_subcommand
	set -l cmd (__fish_qiniu_cdn_manager_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -s c -l config -d '配置文件路径, 如果未传, 从当前目录找配置文件, 如果不存在, 尝试加载`$HOME/.config/qiniu-cdn.toml`' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -s d -l domain -d 'CDN域名, 会覆盖配置文件的domain字段' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -l completion -d '生成shell补全脚本, 支持Bash, Zsh, Fish, PowerShell, Elvish' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -l debug -d '开启debug模式'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -s n -l no-elapsed -d 'Do not print time(ms) elapsed'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "count" -d '查询请求次数'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "diagnostic" -d '诊断疑似IP'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "hitmiss" -d '查询命中率'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "info" -d '查询域名信息'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "ipacl" -d 'IP黑/白名单'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "ipurl" -d '查询IP的URL请求次数'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "isp-count" -d '查询运营商请求次数'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "isp-traffic" -d '查询运营商流量'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "isp-traffic-ratio" -d '查询运营商流量占比'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "log-download" -d '下载请求日志'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "log-filter" -d '过滤请求日志'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "prefetch" -d '文件预取'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "refresh" -d '刷新CDN缓存'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "status" -d '查询状态码'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "top" -d '查询TOP请求'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "traffic" -d '查询计费流量'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -s f -l freq -d '粒度, 可选项为 5min、1hour、1day, 默认1day' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -s r -l region -d '区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -s l -l limit -d '输出条数, 默认全部' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -l five-minute-count -d '每5分钟请求次数告警阈值' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -s n -l no-warn -d '不要发送次数告警'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand count" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -s p -l policy -d '诊断策略，支持T:1:200和C:1:10000这两种格式, 可以用&&和||连接(并用引号引起来)' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -l day -d '诊断结束日, 默认当天, 如2016-07-01' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -s a -l apply-black-ip -d '配置IP黑名单'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -s n -l no-rewrite -d '不覆盖已存在的ip列表，采用追加模式'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -l no-qy-notify -d '不发送企业微信通知'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -l no-prompt -d '不弹出prompt确认'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand diagnostic" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -s f -l freq -d '粒度, 可选项为 5min、1hour、1day, 默认1day' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -s l -l limit -d '输出条数, 默认全部' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand hitmiss" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand info" -s d -l download-ssl-cert -d '下载ssl证书'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand info" -s l -l list-all-domain -d '列出账户下绑定的所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand info" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand info" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipacl" -s i -l ips -d 'ip列表, 多个以英文逗号隔开, 以`d`开头表示移除(模式需保证为no-rewrite)' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipacl" -s w -l white -d '设置白名单'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipacl" -s b -l black -d '设置黑名单'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipacl" -s c -l close -d '关闭黑白名单'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipacl" -s r -l rewrite -d '覆盖已存在的ip列表，默认是追加模式'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipacl" -l no-qy-notify -d '不发送企业微信通知'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipacl" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipacl" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipurl" -s i -l ip -d '要查询的IP, 请求日志一般滞后6个小时左右' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipurl" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipurl" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipurl" -s l -l limit -d '输出条数, 默认全部' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipurl" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand ipurl" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -s f -l freq -d '粒度, 可选项为 5min、1hour、1day, 默认1day' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -s r -l region -d '区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -s l -l limit -d '输出条数, 默认全部' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -l region-sort -d '按区域排序，会查询所有区域并按请求次数从大到小排序'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-count" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -s f -l freq -d '粒度, 可选项为 5min、1hour、1day, 默认1day' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -s r -l regions -d '区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global, 多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -s i -l isp -d 'ISP运营商, 比如all(所有 ISP), telecom(电信), unicom(联通), mobile(中国移动), drpeng(鹏博士), tietong(铁通), cernet(教育网)' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -s l -l limit -d '输出条数, 默认全部' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -l region-sort -d '按区域排序，会查询所有区域并按流量从大到小排序'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -l isp-sort -d '按运营商排序，会查询所有运营商并按流量从大到小排序'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic-ratio" -s r -l regions -d '区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global, 多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic-ratio" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic-ratio" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic-ratio" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic-ratio" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic-ratio" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic-ratio" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand isp-traffic-ratio" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-download" -s d -l day -d '日期, 例如 2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-download" -s l -l limit -d '下载条数, 默认全部' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-download" -l dir -d '下载目录, 默认当前目录的`./logs`目录' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-download" -s n -l no-domain-dir -d '不要将日志放在域名目录里, 和在配置文件设置`download_log_domain_dir=false`同义'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-download" -l unzip-keep -d '解压缩日志并保留源压缩文件'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-download" -l unzip-not-keep -d '解压缩日志不保留源压缩文件'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-download" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-download" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-filter" -s f -l filter-string -d '要过滤的字符串，支持传递多次, 以!!开头表示不包含' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-filter" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-filter" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-filter" -l output-file -d '输出到文件'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-filter" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand log-filter" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand prefetch" -s u -l urls -d '要预取url列表, 总数不超过60条, 多条以英文逗号隔开, 如：http://bar.foo.com/test.zip' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand prefetch" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand prefetch" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand refresh" -s u -l urls -d '要刷新的url列表, 总数不超过60条, 多条以英文逗号隔开, 如：http://bar.foo.com/index.html' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand refresh" -s d -l dirs -d '要刷新的目录url列表, 总数不超过10条, 多条以英文逗号隔开, 如: http://bar.foo.com/dir/' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand refresh" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand refresh" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -s r -l regions -d '区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global, 多个用英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -s l -l limit -d '输出条数, 默认全部' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -s i -l isp -d 'ISP运营商, 比如all(所有 ISP), telecom(电信), unicom(联通), mobile(中国移动), drpeng(鹏博士), tietong(铁通), cernet(教育网)' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -s f -l freq -d '粒度, 可选项为 5min、1hour、1day, 默认1day' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand status" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s r -l region -d '区域, global oversea china beijing...,更多请移步 https://developer.qiniu.com/fusion/4081/cdn-log-analysis#region, 默认global' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s l -l limit -d '输出条数, 默认全部' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s i -l ip -d 'IP模式(默认)'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s u -l url -d 'URL模式'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s t -l traffic -d '根据流量查询(默认)'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s c -l count -d '根据请求数量查询'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand top" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -s s -l start-date -d '开始日期, 例如：2016-07-01, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -s e -l end-date -d '结束日期, 例如：2016-07-03, 默认当天' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -s g -l granularity -d '粒度, 取值：5min/hour/day,  默认day' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -l five-minute-traffic -d '每5分钟流量告警(MB)阈值' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -l domain-exclude -d '排除域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -l domains -d '域名，多个以英文逗号隔开' -r
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -l no-print -d '不要打印流量明细'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -l no-warn -d '不要发送流量告警'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -l all-domain -d '包含所有域名'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -s h -l help -d 'Print help'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand traffic" -s V -l version -d 'Print version'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "count" -d '查询请求次数'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "diagnostic" -d '诊断疑似IP'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "hitmiss" -d '查询命中率'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "info" -d '查询域名信息'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "ipacl" -d 'IP黑/白名单'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "ipurl" -d '查询IP的URL请求次数'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "isp-count" -d '查询运营商请求次数'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "isp-traffic" -d '查询运营商流量'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "isp-traffic-ratio" -d '查询运营商流量占比'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "log-download" -d '下载请求日志'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "log-filter" -d '过滤请求日志'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "prefetch" -d '文件预取'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "refresh" -d '刷新CDN缓存'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "status" -d '查询状态码'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "top" -d '查询TOP请求'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "traffic" -d '查询计费流量'
complete -c qiniu-cdn-manager -n "__fish_qiniu_cdn_manager_using_subcommand help; and not __fish_seen_subcommand_from count diagnostic hitmiss info ipacl ipurl isp-count isp-traffic isp-traffic-ratio log-download log-filter prefetch refresh status top traffic help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
