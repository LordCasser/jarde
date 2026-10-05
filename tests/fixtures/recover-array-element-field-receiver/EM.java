import java.util.HashMap;
import java.util.Map;
public enum EM {
    A("alpha"), B("beta"), C("gamma");
    final String label;
    EM(String l){ this.label = l; }
    static final Map<String,EM> BY_LABEL = new HashMap<String,EM>();
    static {                                                  // enum 静态查找表
        for(EM c : values()){ BY_LABEL.put(c.label, c); }
    }
    static EM of(String l){ return BY_LABEL.get(l); }
    public static void main(String[] a){ System.out.println(""+of("beta")+"/"+of("alpha")+"/"+of("?")); }
}
