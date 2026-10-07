public class SD {
    static String readBack(){                                      // 同型存入 + 读回
        Object[] a = new String[2];
        a[0] = "s";
        return (String) a[0];
    }
    static String objectComponent(){                               // Object 组件（已证兼容）
        Object[] a = new Object[2];
        a[0] = "o";
        return (String) a[0];
    }
    static String nullStore(){                                     // null 值（已证兼容）
        Object[] a = new String[2];
        a[0] = null;
        return "n";
    }
    static String catchWrong(){                                    // 协变存入：捕获 ASE 报类型@点
        Object[] a = new String[2];
        try {
            a[0] = Integer.valueOf(1);
            return "no-throw";
        } catch (ArrayStoreException e) {
            return "wrong@" + e.getClass().getName() + "/" + e.getStackTrace()[0].getMethodName();
        }
    }
    static String catchNumber(){
        Number[] n = new Integer[2];
        try {
            n[0] = Double.valueOf(2.5);
            return "no-throw2";
        } catch (ArrayStoreException e) {
            return "number@" + e.getClass().getName() + "/" + e.getStackTrace()[0].getMethodName();
        }
    }
    static String catchElement(){                                  // 接收者是数组元素读
        Object[][] rows = new String[1][1];
        try {
            rows[0][0] = Integer.valueOf(1);
            return "no-throw3";
        } catch (ArrayStoreException e) {
            return "element@" + e.getClass().getName() + "/" + e.getStackTrace()[0].getMethodName();
        }
    }
    static String primitiveArray(){                                // 基本型数组负例
        int[] a = new int[2];
        a[0] = 7;
        return "p" + a[0];
    }
    static String elementReceiver(){                               // 同型存入（元素接收者对照）
        String[][] rows = new String[1][1];
        rows[0][0] = "r";
        return rows[0][0];
    }
    public static void main(String[] x){
        System.out.println(readBack());
        System.out.println(objectComponent());
        System.out.println(nullStore());
        System.out.println(catchWrong());
        System.out.println(catchNumber());
        System.out.println(catchElement());
        System.out.println(primitiveArray());
        System.out.println(elementReceiver());
    }
}
