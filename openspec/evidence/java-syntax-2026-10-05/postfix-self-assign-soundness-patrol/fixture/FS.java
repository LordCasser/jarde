public class FS {
    int f = 5;
    int fieldSelf(){ f = f++; return f; }                    // 字段版自赋陷阱（getfield/iinc? no—field 用 iadd 路径）
    int fieldPost(){ f++; return f; }                         // 字段语句位（健康对照）
    int compoundSelf(){ int i = 5; i += i++ + 1; return i; }  // 复合+后缀混合
    int fieldTrap2(){ f = f-- ; return f; }
    public static void main(String[] a){ FS x = new FS(); System.out.println(x.fieldSelf()); FS y = new FS(); System.out.println(y.fieldTrap2()); FS z = new FS(); System.out.println(z.compoundSelf()); }
}
