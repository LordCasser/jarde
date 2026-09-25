public class InlineHolder {
 public int value;
 public InlineHolder(int value){InlineOrderEffects.step(5);this.value=value;}
 public int read(int x){InlineOrderEffects.step(4);return value+x;}
}
