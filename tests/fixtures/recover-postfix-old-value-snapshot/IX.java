public class IX {
    static int idx = 0;
    static int[] arr = new int[3];
    static void localWrite(){ int i = 0; int[] a = new int[3]; a[i++] = 10; }        // 局部后缀作下标（写位）
    static int localRead(){ int i = 1; int[] a = {7,8,9}; return a[i--]; }           // 局部后缀作下标（读位）
    static void fieldWrite(){ IX.arr[IX.idx++] = 20; }                               // 静态字段后缀作下标（写位）
    static int fieldRead(){ return IX.arr[IX.idx--]; }                               // 静态字段后缀作下标（读位）
    public static void main(String[] a){
        localWrite();
        int r = localRead();
        fieldWrite();
        int q = fieldRead();
        System.out.println("" + r + "/" + q + "/" + IX.idx + "/" + IX.arr[0] + "/" + IX.arr[2]);
    }
}
