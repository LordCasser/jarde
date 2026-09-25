public class LambdaAdaptationBox {
 private final String value;
 public LambdaAdaptationBox(String value){LambdaAdaptationSupport.calls++;this.value=value;}
 public String value(){return value;}
}
