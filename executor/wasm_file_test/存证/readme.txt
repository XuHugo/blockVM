合约名称
evidence
——————————————————
函数说明：
合约有三个函数：
函数名      功能         参数
init         初始化       evidence_init.json
create   创建存证     evidence_create.json
get       查询存证     evidence_get.json
——————————————————————————
参数说明
init参数是一个字符串，随意填写；
create参数是一组数据，示例中表示的书籍，id、name、description、time；
get通过id获取书籍；
——————————————————————————
说明：
1、时间和id都是人为写进去，只是方便填写；
2、create创建完成之后，会在所有信息后，添加一个owner字段，该字段是一个账户，表示创建该信息的用户；
通过get可以看到完整的信息。
