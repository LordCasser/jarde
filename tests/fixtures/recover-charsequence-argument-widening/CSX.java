public class CSX {
    static <T extends java.io.Serializable> T seal(T t){ return t; }
    static java.io.Serializable sealBuilder(StringBuilder b){ return seal(b); }             // StringBuilder→Serializable 不在封闭 9 行
    static <T extends CharSequence> T cs(T t){ return t; }
    static CharSequence viaSegment(javax.swing.text.Segment s){ return cs(s); }            // Segment→CharSequence 不在封闭 4 行
}
