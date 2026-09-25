import java.lang.reflect.Method;
public class CompareRunner {
    public static void main(String[] args) throws Exception {
        String[] ops = {"eq", "ne", "lt", "le", "gt", "ge", "not_lt"};
        Object[][] values = {
            {Long.MIN_VALUE, -1L, 0L, 1L, Long.MAX_VALUE},
            {Float.NaN, Float.NEGATIVE_INFINITY, -1.0f, -0.0f, 0.0f, 1.0f, Float.POSITIVE_INFINITY, Float.MIN_VALUE, Float.MAX_VALUE},
            {Double.NaN, Double.NEGATIVE_INFINITY, -1.0d, -0.0d, 0.0d, 1.0d, Double.POSITIVE_INFINITY, Double.MIN_VALUE, Double.MAX_VALUE}
        };
        Class<?>[] types = {long.class, float.class, double.class};
        for (int t=0;t<types.length;t++) for (String op:ops) {
            String name=types[t].getName()+"_"+op;
            Method method=CompareProbe.class.getMethod(name,types[t],types[t]);
            for(int a=0;a<values[t].length;a++) for(int b=0;b<values[t].length;b++)
                System.out.println(name+"/"+a+"/"+b+"="+method.invoke(null,values[t][a],values[t][b]));
        }
    }
}
