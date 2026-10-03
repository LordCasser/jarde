public class SCGE<T extends java.lang.Comparable<T>> extends SCGEBase {
    private java.util.List<T> values = new java.util.ArrayList<T>();
    public static void main(java.lang.String[] args) {
        SCGE<java.lang.String> z = new SCGE<java.lang.String>();
        if (z.values != null) { System.out.println("values:true"); } else { System.out.println("values:false"); }
    }
}
class SCGEBase { protected java.util.List values = new java.util.ArrayList(); }
