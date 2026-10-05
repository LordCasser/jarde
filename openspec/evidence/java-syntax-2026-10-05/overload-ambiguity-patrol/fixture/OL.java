public class OL {
    static String pick(int x){ return "int"; }                       // 精确 int
    static String pick(Integer x){ return "boxed"; }                 // 装箱
    static String pick(Object x){ return "object"; }                 // 引用宽化
    static String pick(int... xs){ return "varargs"; }               // varargs（最低优先）
    static String nullRef(String s){ return "string"; }
    static String nullRef(Object o){ return "object"; }              // null 调用点——String 更特化
    static String boxFirst(int x){ return "int"; }
    static String boxFirst(Integer x){ return "boxed"; }
    static String boxFirst(long x){ return "long"; }                 // 宽化 int→long 优先于装箱
    public static void main(String[] a){
        System.out.println(""+pick(5)+"/"+pick((Integer) 5)+"/"+pick((Object) (Integer) 5));
        System.out.println(""+nullRef(null)+"/"+boxFirst(7));
    }
}
