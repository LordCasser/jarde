public class SC {
    static String subtypeStore(){                                  // 组件型的子类型值（未证兼容）
        CharSequence[] cs = new CharSequence[2];
        cs[0] = "s";
        return cs[0].toString();
    }
    public static void main(String[] x){
        System.out.println(subtypeStore());
    }
}
