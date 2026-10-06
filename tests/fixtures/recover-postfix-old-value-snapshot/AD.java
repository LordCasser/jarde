public class AD {
    private Object[] elems = new Object[4]; private int size = 0;
    void add(Object t){ elems[size++] = t; }          // ArrayList.add 核心形（隔离）
    Object get(int i){ return elems[i]; }
    public static void main(String[] a){ AD d = new AD(); d.add("x"); d.add("y"); System.out.println(""+d.get(0)+"/"+d.get(1)); }
}
