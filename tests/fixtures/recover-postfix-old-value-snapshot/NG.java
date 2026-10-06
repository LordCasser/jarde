public class NG {
    static int postSelf(){ int i = 5; i = i++; return i; }                       // 局部自赋陷阱（负例）
    static int postSelfDec(){ int i = 5; i = i--; return i; }                    // 局部自赋陷阱（负例）
    int f = 5;
    int fieldSelf(){ f = f++; return f; }                                        // 字段自赋陷阱（负例）
    int compoundSelf(){ int i = 5; i += i++ + 1; return i; }                     // 多消费方形（负例）
    static int[] xs = {1,0,3};
    static int condShape(){ int i = 0; int n = 0; while(xs[i++] != 0 && i < xs.length){ n++; } return n; }  // B 相条件位（负例）
    public static void main(String[] a){
        NG x = new NG();
        System.out.println("" + postSelf() + "/" + postSelfDec() + "/" + x.fieldSelf() + "/" + x.compoundSelf() + "/" + condShape());
    }
}
