public class CM2 {
    static int immUse(){ int i = 5; int j = i++; return j; }            // 后缀：立即捕获（单语句）
    static int crossStmt(){ int i = 5; i++; int j = i; return j; }      // 拆成两语句（对照——无旧值快照）
    static int postfixExpr(){ int i = 5; return i++ + 10; }             // 后缀在表达式内
    static int prefix(){ int i = 5; return ++i + 10; }                  // 前缀（对照）
    public static void main(String[] a){ System.out.println(""+immUse()+"/"+crossStmt()+"/"+postfixExpr()+"/"+prefix()); }
}
