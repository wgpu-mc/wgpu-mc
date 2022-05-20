package eapi;

import java.util.List;

public class SpectrumLayout<T> {

    private final List<T> layout;

    public SpectrumLayout(List<T> elements) {
        this.layout = List.copyOf(elements);
    }

    public List<T> getElements() {
        return this.layout;
    }

}
