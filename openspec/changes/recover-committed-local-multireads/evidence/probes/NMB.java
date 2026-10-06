public class NMB {
    int tag = 5;
    int add(int x){ return x * 2; }
    NMB self(){ return this; }
    public static void main(String[] a){
        NMB n = new NMB();
        System.out.println(""+n.add(1)+"/"+n.add(2)+"/"+n.self().tag+"/"+(n.self() == n));
    }
}
