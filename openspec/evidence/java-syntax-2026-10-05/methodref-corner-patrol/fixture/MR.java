import java.util.function.*;
public class MR {
    static class Base { String name(){ return "b"; } }
    static class Kid extends Base {
        Supplier<String> boundSuper(){ return super::name; }        // super:: 实例方法
        static Supplier<Kid> ctorRef(){ return Kid::new; }          // 类构造引用（Supplier<Kid>）
    }
    static Function<Integer,int[]> arrRef = int[]::new;             // 数组构造引用
    public static void main(String[] a){
        Kid k = new Kid();
        System.out.println(k.boundSuper().get()+"/"+Kid.ctorRef().get().name()+"/"+arrRef.apply(3).length);
    }
}
