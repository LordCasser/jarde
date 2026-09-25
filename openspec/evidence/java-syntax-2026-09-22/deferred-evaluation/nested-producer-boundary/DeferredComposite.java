public class DeferredComposite {public static int nested(){return DeferredSupport.take(DeferredSupport.value());}public static void keepPool(){DeferredSupport.mark();}}
