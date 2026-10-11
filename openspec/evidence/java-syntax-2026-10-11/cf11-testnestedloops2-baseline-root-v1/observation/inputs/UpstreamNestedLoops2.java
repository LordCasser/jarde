package cf11;
import java.util.List;
public class UpstreamNestedLoops2 {
public boolean test(List<String> list) {
			int j = 0;
			for (int i = 0; i < list.size(); i++) {
				String s = list.get(i);
				while (j < s.length()) {
					j++;
				}
			}
			return j > 10;
		}
}
