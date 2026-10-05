public class AR {
    static int lenInt(Object o){                                      // 原始数组 instanceof + cast
        if(o instanceof int[]){ return ((int[]) o).length; }
        return -1;
    }
    static String firstStr(Object o){                                 // 引用数组 instanceof + cast + 元素消费
        if(o instanceof String[]){ return ((String[]) o)[0]; }
        return "-";
    }
    static String kind(Object o){                                     // 数组/类混合 instanceof 链
        if(o instanceof int[]){ return "int[]"; }
        else if(o instanceof String[]){ return "String[]"; }
        else if(o instanceof Object[]){ return "Object[]"; }
        else if(o instanceof String){ return "String"; }
        return "other";
    }
    static Object covariantUp(int[] xs){ return xs; }                 // 数组协变上转型
    public static void main(String[] a){ System.out.println(""+lenInt(new int[]{1,2,3})+"/"+lenInt("s")+"/"+firstStr(new String[]{"x"})+"/"+kind(new Object[1])+"/"+kind(5)+"/"+(covariantUp(new int[]{7}) instanceof int[])); }
}
