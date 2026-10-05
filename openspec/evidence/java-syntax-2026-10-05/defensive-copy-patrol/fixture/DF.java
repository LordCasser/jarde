public class DF {
    private final int[] data;                                 // ctor 防御性拷贝入字段
    DF(int[] src){ this.data = src.clone(); }
    int[] data(){ return data.clone(); }                      // getter 防御性拷贝出
    int sum(){ int s = 0; for(int v : data){ s += v; } return s; }   // for-each 原生数组
    DF grow(int n){                                            // Arrays.copyOf 扩容链
        int[] bigger = java.util.Arrays.copyOf(data, data.length + n);
        return new DF(bigger);
    }
    static void copyRow(int[][] src, int[] dst, int row){      // arraycopy 行拷贝惯用法
        System.arraycopy(src[row], 0, dst, 0, src[row].length);
    }
    public static void main(String[] a){
        DF d = new DF(new int[]{1,2,3});
        int[] out = d.data();
        out[0] = 99;                                           // 改拷贝不影响内部
        int[][] m = {{7,8,9},{4,5,6}};
        int[] row = new int[3];
        copyRow(m, row, 1);
        System.out.println(""+d.sum()+"/"+d.data()[0]+"/"+d.grow(2).sum()+"/"+row[0]+row[1]+row[2]);
    }
}
