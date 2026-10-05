public class ES {
    static String tabNewline(){ return "a\tb\nc"; }                  // \t \n
    static String quotes(){ return "say \"hi\" and 'bye'"; }         // 嵌套引号
    static String backslash(){ return "C:\\dir\\file"; }             // 反斜杠
    static String octal(){ return "\170\172"; }                      // 八进制转义 (xz)
    static String unicode(){ return "\u4e2d\u6587"; }                // Unicode 转义 (中文)
    static String mixed(){ return "\r\n\t\\\"\u00e9"; }              // 混合
    public static void main(String[] a){ System.out.println(""+tabNewline().replace('\n','|')+"/"+quotes()+"/"+backslash()+"/"+octal()+"/"+unicode()+"/"+mixed().replace('\r','R').replace('\n','N')); }
}
