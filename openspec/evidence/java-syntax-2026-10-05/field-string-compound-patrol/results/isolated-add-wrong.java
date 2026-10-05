public class FD {
    String field;
    FD(String f){ field = f; }
    FD add(String arg1){
        return this;   // jarde 渲染体（剥离注释）
    }
    public static void main(String[] a){ System.out.println(new FD("f").add("x").add("y").field); }
}
