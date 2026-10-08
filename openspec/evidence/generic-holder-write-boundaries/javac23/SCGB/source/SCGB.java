import java.util.HashMap;
import java.util.List;
import java.util.Map;
public class SCGB<T extends Comparable<T>> {
    private Map<String,List<T>> index = new HashMap<String,List<T>>();
    public static void main(String[] args) { SCGB<String> z=new SCGB<String>(); System.out.println("index:"+(z.index!=null)); Object read=z.index; System.out.println("read:"+(read instanceof Map)); }
}
