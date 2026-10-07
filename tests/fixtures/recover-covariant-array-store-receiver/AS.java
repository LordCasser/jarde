public class AS {
    static String storeWrong(){                                    // 协变数组存入异型 → ArrayStoreException
        Object[] a = new String[2];
        a[0] = Integer.valueOf(1);                                 // 编译通过、运行抛
        return "unreachable";
    }
    static String storeRight(){                                    // 同型存入（对照）
        Object[] a = new String[2];
        a[0] = "s";
        return (String) a[0];
    }
    static String storeNumber(){                                   // Number[]←Integer[] 协变 + Double 存入
        Number[] n = new Integer[2];
        n[0] = Double.valueOf(2.5);
        return "unreachable2";
    }
    public static void main(String[] x){
        System.out.println(storeRight());
        try { System.out.println(storeWrong()); } catch(ArrayStoreException e){ System.out.println("ASE1"); }
        try { System.out.println(storeNumber()); } catch(ArrayStoreException e){ System.out.println("ASE2"); }
    }
}
