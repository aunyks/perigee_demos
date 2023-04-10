import {
  Mesh,
  CylinderGeometry,
  ShaderMaterial,
  Uniform,
} from '/js/graphics/three.module.js'

class MarkerCylinder extends Mesh {
  constructor(radius, height, color) {
    super(
      new CylinderGeometry(radius, radius, height, 20, 1),
      new ShaderMaterial({
        transparent: true,
        uniforms: {
          color: new Uniform(color),
        },
        vertexShader: `
        varying vec2 vUv;
        uniform vec3 color;

        void main() {
          vUv = uv;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }`,
        fragmentShader: `
        varying vec2 vUv;
        uniform vec3 color;

        void main() {
          gl_FragColor = vec4(color, smoothstep(0.0, 1.0, 1.0 - vUv.y));
        }`,
      })
    )
  }
}

export default MarkerCylinder
