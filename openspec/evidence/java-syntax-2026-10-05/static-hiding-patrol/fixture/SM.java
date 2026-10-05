public class SM {
    static class Base { static String who(){ return "base"; } static String shared(){ return "base-shared"; } }
    static class Kid extends Base { static String who(){ return "kid"; } }   // 隐藏（非覆盖——静态）
    static String viaBase(){ return Base.who() + "/" + Base.shared(); }
    static String viaKid(){ return Kid.who() + "/" + Kid.shared(); }         // shared 经 Kid 限定（继承静态）
    static String viaInstance(Base b){ return b.getClass().getSimpleName(); } // 实例上的类信息
    interface IF { static String helper(){ return "if-static"; } default String d(){ return helper() + "-d"; } } // 接口静态方法被 default 消费
    static String useIf(){ return new IF(){ public String dummy(){ return ""; } }.d(); }
    public static void main(String[] a){ System.out.println(""+viaBase()+"/"+viaKid()+"/"+useIf()); }
}
