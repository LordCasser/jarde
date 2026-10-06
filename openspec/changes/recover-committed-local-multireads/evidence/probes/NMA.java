public class NMA {
    int tag = 5;
    int add(int x){ return x * 2; }
    NMA self(){ return this; }
    public static void main(String[] a){
        NMA n = new NMA();
        System.out.println(""+n.add(1)+"/"+n.add(2)+"/"+n.self().tag+"/"+n.tag);
    }
}
