public class DeferredScope {
 public static int direct(){try{return ScopeSupport.value();}catch(IllegalStateException ex){return 7;}}
 public static void keepPool(){ScopeSupport.mark();}
}
