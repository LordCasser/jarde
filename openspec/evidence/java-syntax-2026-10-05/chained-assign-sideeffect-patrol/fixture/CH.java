public class CH {
    static int a, b, c;
    static void chain(){ CH.a = CH.b = CH.c = 5; }                      // 链式赋值（字段）
    static int chainLocal(){ int x, y; x = y = 7; return x + y; }       // 链式（局部）
    static int[] arr = new int[3];
    static int idx = 0;
    static void sideIdx(){ CH.arr[CH.idx++] = 10; CH.arr[CH.idx++] = 20; }  // 副作用下标（i++ 在 index 位）
    static int sideRead(){ return CH.arr[CH.idx--]; }                   // 读位副作用下标
    public static void main(String[] args){ chain(); sideIdx(); System.out.println(""+a+"/"+b+"/"+c+"/"+chainLocal()+"/"+arr[0]+"/"+arr[1]+"/"+idx+"/"+sideRead()+"/"+idx); }
}
