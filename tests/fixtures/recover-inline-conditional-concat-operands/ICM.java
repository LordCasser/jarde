public class ICM {
    int tag = 5;
    int add(int x){ return x * 2; }
    ICM self(){ return this; }
    public static void main(String[] a){
        ICM n = new ICM();
        System.out.println(""+n.add(1)+"/"+n.add(2)+"/"+n.self().tag+"/"+n.tag);
    }
}
