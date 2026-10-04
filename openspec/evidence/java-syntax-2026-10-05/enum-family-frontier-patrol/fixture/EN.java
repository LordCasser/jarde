public class EN {
    enum Plain { A, B, C }                                  // 简单枚举
    enum WithBody {                                          // 带体枚举
        X { void extra(){ System.out.println("X-extra"); } },
        Y;
        void extra(){ System.out.println("default"); }
    }
    enum Impl implements Runnable {                          // 枚举实现接口
        R1, R2;
        public void run(){ System.out.println("run:"+name()); }
    }
    enum WithCtor {                                          // 带构造器与字段
        BIG(10), SMALL(1);
        final int size;
        WithCtor(int s){ size = s; }
    }
    public static void main(String[] a){
        System.out.println(Plain.A+"/"+Plain.values().length+"/"+WithBody.values().length);
        WithBody.X.extra(); Impl.R1.run(); System.out.println(WithCtor.BIG.size+"/"+WithCtor.values()[1]);
    }
}
