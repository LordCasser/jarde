public class PC {
    static void check(String name, int age, Object extra){                    // Commons Validate 风格连环校验
        if(name == null){ throw new IllegalArgumentException("name is null"); }
        if(name.isEmpty()){ throw new IllegalArgumentException("name is empty"); }
        if(age < 0){ throw new IllegalArgumentException("age: " + age); }
        if(age > 150){ throw new IllegalArgumentException("age too big: " + age + " for " + name); }
        if(extra == null){ throw new NullPointerException("extra required"); }
    }
    static String fmt(String tpl, Object v){                                   // 消息工厂（SB + String.valueOf）
        return new StringBuilder(tpl.length() + 16).append(tpl).append('=').append(v).toString();
    }
    public static void main(String[] a){
        try { check(null, 1, new Object()); } catch(IllegalArgumentException e){ System.out.println("A:"+e.getMessage()); }
        try { check("x", -1, new Object()); } catch(IllegalArgumentException e){ System.out.println("B:"+e.getMessage()); }
        try { check("bob", 200, new Object()); } catch(IllegalArgumentException e){ System.out.println("C:"+e.getMessage()); }
        try { check("ok", 1, null); } catch(NullPointerException e){ System.out.println("D:"+e.getMessage()); }
        System.out.println("fmt:"+fmt("count", 42));
    }
}
