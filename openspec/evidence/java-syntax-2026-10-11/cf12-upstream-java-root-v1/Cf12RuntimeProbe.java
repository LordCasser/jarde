import java.lang.reflect.*;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
public final class Cf12RuntimeProbe {
    static String value(Object result) {
        if (result == null) return "null";
        if (result instanceof String) return "s:" + Base64.getEncoder().encodeToString(((String) result).getBytes(StandardCharsets.UTF_8));
        return result.getClass().getSimpleName()+":"+result;
    }
    static Object invoke(Class<?> cls, Object self, String name, Class<?>[] types, Object... args) throws Exception {
        return cls.getMethod(name,types).invoke(self,args);
    }
    public static void main(String[] args) throws Exception {
        Class<?> cls = Class.forName(args[0]);
        Object self = cls.getConstructor().newInstance();
        switch(args[1]) {
            case "TestSwitch":
                for(String s:new String[]{"", ".", "/", "]", "?", "a", "./]?a", "Java/Hello.class", "中/文?", "\u0000", "\uFFFF"}) {
                    System.out.println("char:"+value(s)+"="+value(invoke(cls,self,"test",new Class<?>[]{String.class},s)));
                }
                break;
            case "TestSwitchNoDefault":
                for(int a:new int[]{Integer.MIN_VALUE,-1,0,1,2,3,4,5,Integer.MAX_VALUE}) {
                    System.out.println("no-default:"+a);invoke(cls,self,"test",new Class<?>[]{int.class},a);
                }
                break;
            case "TestSwitchLabels":
                Class<?> inner=Class.forName(args[0]+"$Inner");Object in=inner.getConstructor().newInstance();
                for(int a:new int[]{Integer.MIN_VALUE,-1,0,2747,2748,2749,3293,3294,3295,Integer.MAX_VALUE}) {
                    System.out.println("labels:"+a+":"+value(invoke(cls,null,"f1",new Class<?>[]{int.class},a))+":"+value(invoke(inner,in,"f1",new Class<?>[]{int.class},a)));
                }
                break;
            case "TestSwitchFallThrough":
                invoke(cls,self,"check",new Class<?>[]{});
                for(int a:new int[]{Integer.MIN_VALUE,-1,0,1,2,3,Integer.MAX_VALUE})System.out.println("fall:"+a+":"+value(invoke(cls,self,"testWrap",new Class<?>[]{int.class},a)));
                break;
            case "TestSwitchWithFallThroughCase":
                invoke(cls,self,"check",new Class<?>[]{});
                for(int a=-5;a<=9;a++)for(boolean b:new boolean[]{false,true})for(boolean c:new boolean[]{false,true})
                    System.out.println("conditional-fall:"+a+":"+b+":"+c+":"+value(invoke(cls,self,"test",new Class<?>[]{int.class,boolean.class,boolean.class},a,b,c)));
                break;
            default:throw new AssertionError(args[1]);
        }
    }
}
