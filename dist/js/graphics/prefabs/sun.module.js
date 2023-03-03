import {
  PlaneGeometry,
  ShaderMaterial,
  Uniform,
  Object3D,
  Color,
  Mesh,
  Group,
} from '/js/graphics/three.module.js'

class Sun extends Group {
  constructor(_color, _scaleFactor) {
    super()
    const scaleFactor = _scaleFactor || 1
    const color = (_color || new Color(0xeeeeee)).convertLinearToSRGB()
    this._sunMesh = new Mesh(
      new PlaneGeometry(100, 100),
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
            gl_FragColor = vec4(color, 1.0 - smoothstep(0.0, .5, distance(vUv, vec2(0.5))));
          }
          `,
      })
    )
    this.pivot = new Object3D()

    this.pivot.add(this._sunMesh)
    this.add(this.pivot)

    this._sunMesh.position.set(0, 0, -700)
    this._sunMesh.lookAt(0, 0, 0)
    this._sunMesh.scale.set(scaleFactor, scaleFactor, scaleFactor)

    // - 1 so that it's in front of the skydome
    this._sunMesh.renderOrder = -(Number.MAX_SAFE_INTEGER - 1)
    this.renderOrder = -(Number.MAX_SAFE_INTEGER - 1)
  }
}

export default Sun
