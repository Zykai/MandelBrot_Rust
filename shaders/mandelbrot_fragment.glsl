#version 330 core
out vec4 FragColor;

in vec2 position;

uniform int maxIterations;

int mandelbrot(float x0, float y0){
    float x = 0;
    float y = 0;
    int iteration = 0;
    while (x * x + y * y <= 2*2 && iteration < maxIterations){
        float xTemp = x*x - y*y + x0;
        y = 2*x*y + y0;
        x = xTemp;
        iteration = iteration + 1;
    }
    return iteration;
}

void main() {
    int iterations = mandelbrot(position.x, position.y);
    if (iterations == maxIterations) {
        FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0);
    } else if (iterations > maxIterations * 0.9) {
        FragColor = vec4(0.000, 0.000, 0.804, 1.0);
    } else if (iterations > maxIterations * 0.8) {
        FragColor = vec4(0.196, 0.804, 0.196, 1.0);
    } else if (iterations > maxIterations * 0.7) {
        FragColor = vec4(0.125, 0.698, 0.667, 1.0);
    } else if (iterations > maxIterations * 0.6) {
        FragColor = vec4(0.855, 0.647, 0.125, 1.0);
    } else if (iterations > maxIterations * 0.5) {
        FragColor = vec4(0.663, 0.663, 0.663, 1.0);
    } else if (iterations > maxIterations * 0.4) {
        FragColor = vec4(0.502, 0.000, 0.000, 1.0);
    } else if (iterations > maxIterations * 0.4) {
        FragColor = vec4(0.000, 1.000, 1.000, 1.0);
    } else {
        discard;
    }
}

