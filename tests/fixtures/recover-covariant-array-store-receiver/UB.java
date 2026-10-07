public class UB {
    static String merged(String[] a, Integer[] b, boolean f){      // 合并局部：组件型无事实
        Object[] x;
        if (f) {
            x = a;
        } else {
            x = b;
        }
        x[0] = Integer.valueOf(1);
        return "u";
    }
    static String throughObject(Object o){                         // checkcast 接收者：组件型可证
        ((String[]) o)[0] = "s";
        return "v";
    }
    public static void main(String[] y){
        System.out.println(merged(new String[1], new Integer[1], true));
        System.out.println(throughObject(new String[1]));
    }
}
