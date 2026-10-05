public class AC {
    static String[] src = {"a", "b", null};
    static int pos = 0;
    static String read(){ return pos < src.length ? src[pos++] : null; }   // 模拟 reader
    static int ioLoop(){                                                    // 经典 while((line=read())!=null)
        int n = 0;
        String line;
        while((line = read()) != null){ n++; }
        return n;
    }
    static int forAssign(int[] xs){                                          // for 条件位赋值
        int s = 0; int i; int[] a = xs;
        for(i = 0; i < a.length; ){ s += a[i]; i++; }
        return s;
    }
    static boolean condAssign(boolean[] bs){                                 // if 条件位赋值
        boolean any = false; boolean v;
        for(int i = 0; i < bs.length; i++){ if((v = bs[i]) == true){ any = true; } }
        return any;
    }
    public static void main(String[] a){ System.out.println(""+ioLoop()+"/"+forAssign(new int[]{1,2,3})+"/"+condAssign(new boolean[]{false,true})); }
}
