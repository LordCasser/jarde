public class ICB {
    int tag = 5;
    int add(int x){ return x * 2; }
    ICB self(){ return this; }
    public static void main(String[] a){
        ICB n = new ICB();
        System.out.println(""+n.add(1)+"/"+n.add(2)+"/"+n.self().tag+"/"+(n.self() == n));
    }
}
