// github.com/chipweinberger/ShaderToyLite.js/blob/main/ShaderToyLite.js

import { create_singular_request_animation_frame } from '#lib/helpers.ts'

type Key = 'A' | 'B' | 'C' | 'D' | 'Image'

const possible_keys = ['A', 'B', 'C', 'D', 'Image'] as const

type Locations = {
	iResolution: WebGLUniformLocation | null
	iTime: WebGLUniformLocation | null
	iTimeDelta: WebGLUniformLocation | null
	iFrameRate: WebGLUniformLocation | null
	iFrame: WebGLUniformLocation | null
	iChannelTime: WebGLUniformLocation | null
	iChannelResolution: WebGLUniformLocation | null
	iChannel0: WebGLUniformLocation | null
	iChannel1: WebGLUniformLocation | null
	iChannel2: WebGLUniformLocation | null
	iChannel3: WebGLUniformLocation | null
	iMouse: WebGLUniformLocation | null
	iDate: WebGLUniformLocation | null
	iSampleRate: WebGLUniformLocation | null
	iStream: WebGLUniformLocation | null
	iVolume: WebGLUniformLocation | null
	vertexInPosition: number
}

export type ShaderToyLite = NonNullable<ReturnType<typeof shader_toy_lite>['data']>

export function shader_toy_lite(canvas: HTMLCanvasElement) {
	const hdr = `#version 300 es
    #ifdef GL_ES
    precision highp float;
    precision highp int;
    precision mediump sampler3D;
    #endif
    #define texture2D texture
    uniform vec3      iResolution;           // viewport resolution (in pixels)
    uniform float     iTime;                 // shader playback time (in seconds)
    uniform float     iTimeDelta;            // render time (in seconds)
    uniform float     iFrameRate;            // shader frame rate
    uniform int       iFrame;                // shader playback frame
    uniform float     iChannelTime[4];       // channel playback time (in seconds)
    uniform vec3      iChannelResolution[4]; // channel resolution (in pixels)
    uniform vec4      iMouse;                // mouse pixel coords. xy: current (if MLB down), zw: click
    uniform sampler2D iChannel0;             // input channel 0
    uniform sampler2D iChannel1;             // input channel 1
    uniform sampler2D iChannel2;             // input channel 2
    uniform sampler2D iChannel3;             // input channel 3
    uniform vec4      iDate;                 // (year, month, day, unixtime in seconds)
    uniform float     iSampleRate;           // sound sample rate (i.e., 44100)
    uniform float     iStream;               // audio stream value
    uniform float     iVolume;               // audio volume value
    out vec4          frag_out_color;
    void mainImage( out vec4 c, in vec2 f );
    void main( void )
    {
        vec4 color = vec4(0.0,0.0,0.0,0.0);
        mainImage( color, gl_FragCoord.xy );
        frag_out_color = vec4(color);
    }
    `

	const basic_vertex_shader = `#version 300 es
    #ifdef GL_ES
    precision highp float;
    precision highp int;
    precision mediump sampler3D;
    #endif
    in vec2 vertexInPosition;
    void main() {
        gl_Position = vec4(vertexInPosition, 0.0, 1.0);
    }
    `

	const quad_vertices = new Float32Array([
		-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0,
	])

	const opts = {
		alpha: false,
		depth: false,
		stencil: false,
		premultiplied_alpha: false,
		antialias: true,
		preserve_drawing_buffer: false,
		power_preference: 'high-performance',
	}

	const retrieved_gl = canvas.getContext('webgl2', opts)
	if (!(retrieved_gl instanceof WebGL2RenderingContext)) {
		return { data: null, error: 'WebGL2 is not supported' }
	}
	const gl = retrieved_gl

	// timing
	let is_playing = false
	let first_draw_time = 0
	let prev_draw_time = 0

	// callback
	let on_draw_callback: (() => void) | undefined

	// uniforms
	let iframe = 0
	const imouse = { x: 0, y: 0, clickX: 0, clickY: 0 }
	let istream = 0
	let ivolume = 0

	// shader common source
	let common = ''

	// render passes variables. valid keys:
	//   'A', 'B', 'C', 'D', 'Image'
	const sourcecode: Partial<Record<Key, string>> = {} // fragment shader code
	const ichannels: Partial<Record<Key, Record<number, Key>>> = {} // texture inputs
	const atexture: Partial<Record<Key, WebGLTexture | null>> = {} // front texture (input/output)
	const btexture: Partial<Record<Key, WebGLTexture | null>> = {} // back texture  (input/output)
	const aframebuf: Partial<Record<Key, WebGLFramebuffer | null>> = {} // front buffer (output)
	const bframebuf: Partial<Record<Key, WebGLFramebuffer | null>> = {} // back buffer (output)
	const program: Partial<Record<Key, WebGLProgram | null>> = {} // webgl program
	const fragment_shaders: Partial<Record<Key, WebGLShader | null>> = {}
	const vertex_shaders: Partial<Record<Key, WebGLShader | null>> = {}
	const location: Partial<Record<Key, Partial<Locations>>> = {} // uniform location
	const flip: Partial<Record<Key, boolean>> = {} // a b flip
	let quad_buffer: WebGLBuffer | null // two full screen triangles

	const setup = () => {
		gl.getExtension('OES_texture_float_linear')
		gl.getExtension('OES_texture_half_float_linear')
		gl.getExtension('EXT_color_buffer_float')
		gl.getExtension('WEBGL_debug_shaders')
		possible_keys.forEach((key) => {
			sourcecode[key] = ''
			ichannels[key] = {}
			program[key] = null
			location[key] = {}
			if (key !== 'Image') {
				atexture[key] = create_texture()
				btexture[key] = create_texture()
				aframebuf[key] = create_frame_buffer(atexture[key])
				bframebuf[key] = create_frame_buffer(btexture[key])
				flip[key] = false
			}
		})

		// bind the geometry
		quad_buffer = gl.createBuffer()
		gl.bindBuffer(gl.ARRAY_BUFFER, quad_buffer)
		gl.bufferData(gl.ARRAY_BUFFER, quad_vertices, gl.STATIC_DRAW)

		// Set viewport size
		gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)

		window.addEventListener('resize', function () {
			gl.canvas.width = canvas.width
			gl.canvas.height = canvas.height
			gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)
		})

		canvas.addEventListener('mousemove', (event) => {
			imouse.x = event.offsetX
			imouse.y = canvas.height - event.offsetY
		})

		canvas.addEventListener('mousedown', (event) => {
			imouse.clickX = event.offsetX
			imouse.clickY = canvas.height - event.offsetY
		})

		canvas.addEventListener('mouseup', () => {
			imouse.clickX = 0
			imouse.clickY = 0
		})
	}

	const create_texture = () => {
		const texture = gl.createTexture()
		gl.bindTexture(gl.TEXTURE_2D, texture)
		gl.texImage2D(
			gl.TEXTURE_2D,
			0,
			gl.RGBA32F,
			gl.canvas.width,
			gl.canvas.height,
			0,
			gl.RGBA,
			gl.FLOAT,
			null,
		)
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR)
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR)
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE)
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE)
		return texture
	}

	const create_frame_buffer = (texture: WebGLTexture) => {
		const framebuffer = gl.createFramebuffer()
		gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer)
		gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0)
		gl.bindTexture(gl.TEXTURE_2D, null)
		gl.bindFramebuffer(gl.FRAMEBUFFER, null)
		return framebuffer
	}

	const compile_program = (key: Key) => {
		// Delete previous program + shaders if they exist
		if (program[key]) {
			gl.deleteProgram(program[key])
			program[key] = null
		}
		if (vertex_shaders[key]) {
			gl.deleteShader(vertex_shaders[key])
			vertex_shaders[key] = null
		}
		if (fragment_shaders[key]) {
			gl.deleteShader(fragment_shaders[key])
			fragment_shaders[key] = null
		}

		const vert = gl.createShader(gl.VERTEX_SHADER)
		if (!vert) {
			return { data: null, error: 'Failed to create vertex shader' }
		}
		gl.shaderSource(vert, basic_vertex_shader)
		gl.compileShader(vert)

		if (!gl.getShaderParameter(vert, gl.COMPILE_STATUS)) {
			gl.deleteShader(vert)
			return { data: null, error: 'Vertex Shader compilation failed: ' + gl.getShaderInfoLog(vert) }
		}

		const frag_source = hdr + common + sourcecode[key]
		const frag = gl.createShader(gl.FRAGMENT_SHADER)
		if (!frag) {
			return { error: 'Failed to create fragment shader' }
		}
		gl.shaderSource(frag, frag_source)
		gl.compileShader(frag)

		if (!gl.getShaderParameter(frag, gl.COMPILE_STATUS)) {
			console.error('Fragment Shader compilation failed: ' + gl.getShaderInfoLog(frag))
			console.error(frag_source)
			gl.deleteShader(frag)
			return null
		}

		const new_program = gl.createProgram()
		gl.attachShader(new_program, vert)
		gl.attachShader(new_program, frag)
		gl.linkProgram(new_program)

		gl.deleteShader(vert)
		gl.deleteShader(frag)

		if (!gl.getProgramParameter(new_program, gl.LINK_STATUS)) {
			return {
				data: null,
				error: 'Program initialization failed: ' + gl.getProgramInfoLog(new_program),
			}
		}

		if (!location[key]) {
			return { data: null, error: 'Unexpected missing location[key]' }
		}

		// uniform locations
		location[key]['iResolution'] = gl.getUniformLocation(new_program, 'iResolution')
		location[key]['iTime'] = gl.getUniformLocation(new_program, 'iTime')
		location[key]['iTimeDelta'] = gl.getUniformLocation(new_program, 'iTimeDelta')
		location[key]['iFrameRate'] = gl.getUniformLocation(new_program, 'iFrameRate')
		location[key]['iFrame'] = gl.getUniformLocation(new_program, 'iFrame')
		location[key]['iChannelTime'] = gl.getUniformLocation(new_program, 'iChannelTime[0]')
		location[key]['iChannelResolution'] = gl.getUniformLocation(
			new_program,
			'iChannelResolution[0]',
		)
		location[key]['iChannel0'] = gl.getUniformLocation(new_program, 'iChannel0')
		location[key]['iChannel1'] = gl.getUniformLocation(new_program, 'iChannel1')
		location[key]['iChannel2'] = gl.getUniformLocation(new_program, 'iChannel2')
		location[key]['iChannel3'] = gl.getUniformLocation(new_program, 'iChannel3')
		location[key]['iMouse'] = gl.getUniformLocation(new_program, 'iMouse')
		location[key]['iDate'] = gl.getUniformLocation(new_program, 'iDate')
		location[key]['iSampleRate'] = gl.getUniformLocation(new_program, 'iSampleRate')
		location[key]['iStream'] = gl.getUniformLocation(new_program, 'iStream')
		location[key]['iVolume'] = gl.getUniformLocation(new_program, 'iVolume')
		location[key]['vertexInPosition'] = gl.getAttribLocation(new_program, 'vertexInPosition')

		vertex_shaders[key] = vert
		fragment_shaders[key] = frag
		return new_program
	}

	const repeat = (times: number, arr: number[]) => {
		let result: number[] = []
		for (let i = 0; i < times; i++) {
			result = [...result, ...arr]
		}
		return result
	}

	type ShaderConfig = {
		source?: string
		iChannel0?: Key
		iChannel1?: Key
		iChannel2?: Key
		iChannel3?: Key
	}

	const set_shader = (config: ShaderConfig, key: Key) => {
		// Unbind before deleting
		gl.bindTexture(gl.TEXTURE_2D, null)
		gl.bindFramebuffer(gl.FRAMEBUFFER, null)
		if (atexture[key]) gl.deleteTexture(atexture[key])
		if (btexture[key]) gl.deleteTexture(btexture[key])
		if (aframebuf[key]) gl.deleteFramebuffer(aframebuf[key])
		if (bframebuf[key]) gl.deleteFramebuffer(bframebuf[key])

		atexture[key] = null
		btexture[key] = null
		aframebuf[key] = null
		bframebuf[key] = null

		if (config) {
			if (config.source) {
				sourcecode[key] = config.source
				program[key] = compile_program(key)
				if (!program[key]) {
					console.error('Failed to compile ' + key)
				}
			}

			// Recreate textures/framebuffers for buffers
			if (key !== 'Image') {
				atexture[key] = create_texture()
				btexture[key] = create_texture()
				aframebuf[key] = create_frame_buffer(atexture[key])
				bframebuf[key] = create_frame_buffer(btexture[key])
				flip[key] = false
			}

			for (const i of [0, 1, 2, 3] as const) {
				const s = config[`iChannel${i}`]
				if (s === 'A' || s === 'B' || s === 'C' || s === 'D') {
					if (!ichannels[key]) {
						return { error: 'Unexpected missing ichannels[key]' }
					}
					ichannels[key][i] = s
				}
			}
		} else {
			sourcecode[key] = ''
			program[key] = null
		}
	}

	const draw = () => {
		// current time
		const now = is_playing ? Date.now() : prev_draw_time
		const date = new Date(now)

		// first draw?
		if (first_draw_time === 0) {
			first_draw_time = now
		}

		// call callback
		if (on_draw_callback) {
			on_draw_callback()
		}

		// time difference between frames in seconds
		const itime_delta = (now - prev_draw_time) * 0.001

		// time in seconds
		const itime = (now - first_draw_time) * 0.001
		const idate = [date.getFullYear(), date.getMonth(), date.getDate(), date.getTime() * 0.001]

		// channel uniforms
		const ichannel_times = new Float32Array(repeat(4, [itime]))
		const ichannel_resolutions = new Float32Array(repeat(4, [gl.canvas.width, gl.canvas.height, 0]))

		for (const key of possible_keys) {
			if (program[key]) {
				// framebuffer
				if (key === 'Image') {
					gl.bindFramebuffer(gl.FRAMEBUFFER, null)
				} else {
					const output = flip[key] ? bframebuf[key] : aframebuf[key]
					gl.bindFramebuffer(gl.FRAMEBUFFER, output ?? null)
				}

				// textures
				for (let i = 0; i < 4; i++) {
					if (!ichannels[key]) {
						return { error: 'Unexpected missing ichannels[key]' }
					}
					const chkey = ichannels[key][i]
					if (chkey) {
						const input = flip[chkey] ? atexture[chkey] : btexture[chkey]
						if (!gl[`TEXTURE${i}` as 'TEXTURE0']) {
							return { error: `Unexpected missing gl.TEXTURE${i}` }
						}
						gl.activeTexture(gl[`TEXTURE${i}` as 'TEXTURE0'])
						gl.bindTexture(gl.TEXTURE_2D, input ?? null)
					}
				}

				// program
				gl.useProgram(program[key])

				if (!location[key]) {
					return { error: 'Unexpected missing location[key]' }
				}

				// uniforms
				gl.uniform3f(location[key]['iResolution'] ?? null, gl.canvas.width, gl.canvas.height, 1.0)
				gl.uniform1f(location[key]['iTime'] ?? null, itime)
				gl.uniform1f(location[key]['iTimeDelta'] ?? null, itime_delta)
				gl.uniform1f(location[key]['iFrameRate'] ?? null, 60)
				gl.uniform1i(location[key]['iFrame'] ?? null, iframe)
				gl.uniform1fv(location[key]['iChannelTime'] ?? null, ichannel_times)
				gl.uniform3fv(location[key]['iChannelResolution'] ?? null, ichannel_resolutions)
				gl.uniform1i(location[key]['iChannel0'] ?? null, 0)
				gl.uniform1i(location[key]['iChannel1'] ?? null, 1)
				gl.uniform1i(location[key]['iChannel2'] ?? null, 2)
				gl.uniform1i(location[key]['iChannel3'] ?? null, 3)
				gl.uniform4f(
					location[key]['iMouse'] ?? null,
					imouse.x,
					imouse.y,
					imouse.clickX,
					imouse.clickY,
				)
				gl.uniform4f(location[key]['iDate'] ?? null, idate[0], idate[1], idate[2], idate[3])
				gl.uniform1f(location[key]['iSampleRate'] ?? null, 44100)
				gl.uniform1f(location[key]['iStream'] ?? null, istream)
				gl.uniform1f(location[key]['iVolume'] ?? null, ivolume)

				// viewport
				gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)

				// vertexs
				gl.bindBuffer(gl.ARRAY_BUFFER, quad_buffer)
				if (location[key]['vertexInPosition'] === undefined) {
					return { error: 'Unexpected missing location[key].vertexInPosition' }
				}
				gl.vertexAttribPointer(location[key]['vertexInPosition'], 2, gl.FLOAT, false, 0, 0)
				gl.enableVertexAttribArray(location[key]['vertexInPosition'])

				// draw
				gl.drawArrays(gl.TRIANGLES, 0, 6)

				flip[key] = !flip[key]
			}
		}

		// time of last draw
		prev_draw_time = now

		// frame counter
		iframe++
	}

	const raf = create_singular_request_animation_frame()

	// Animation loop
	const animate = () => {
		if (is_playing) {
			draw()
			raf(animate)
		}
	}

	const set_common = (source: string) => {
		if (source === undefined) {
			source = ''
		}
		if (source === null) {
			source = ''
		}
		common = source
		possible_keys.forEach((key) => {
			if (program[key]) {
				program[key] = compile_program(key)
			}
		})
	}

	const set_buffer_a = (config: ShaderConfig) => {
		set_shader(config, 'A')
	}

	const set_buffer_b = (config: ShaderConfig) => {
		set_shader(config, 'B')
	}

	const set_buffer_c = (config: ShaderConfig) => {
		set_shader(config, 'C')
	}

	const set_buffer_d = (config: ShaderConfig) => {
		set_shader(config, 'D')
	}

	const set_image = (config: ShaderConfig) => {
		set_shader(config, 'Image')
	}

	const set_on_draw = (callback: (() => void) | undefined) => {
		on_draw_callback = callback
	}

	const set_stream = (value: number) => {
		istream = value
	}

	const set_volume = (value: number) => {
		ivolume = value
	}

	const add_texture = (texture: WebGLTexture, key: Key) => {
		if (atexture[key]) {
			gl.deleteTexture(atexture[key])
			atexture[key] = null
		}
		if (btexture[key]) {
			gl.deleteTexture(btexture[key])
			btexture[key] = null
		}
		if (aframebuf[key]) {
			gl.deleteFramebuffer(aframebuf[key])
			aframebuf[key] = null
		}
		if (bframebuf[key]) {
			gl.deleteFramebuffer(bframebuf[key])
			bframebuf[key] = null
		}
		atexture[key] = texture
		btexture[key] = texture
		flip[key] = false
	}

	const time = () => {
		return (prev_draw_time - first_draw_time) * 0.001
	}

	const reset = () => {
		const now = Date.now()
		first_draw_time = now
		prev_draw_time = now
		iframe = 0
		draw()
	}

	const pause = () => {
		is_playing = false
	}

	const play = () => {
		if (!is_playing) {
			is_playing = true
			const now = Date.now()
			const elapsed = prev_draw_time - first_draw_time
			first_draw_time = now - elapsed
			prev_draw_time = now
			animate()
		}
	}

	const recreate_textures = () => {
		for (const key of ['A', 'B', 'C', 'D'] as const) {
			if (atexture[key]) {
				// Delete old textures
				gl.deleteTexture(atexture[key])
				gl.deleteTexture(btexture[key] ?? null)
				gl.deleteFramebuffer(aframebuf[key] ?? null)
				gl.deleteFramebuffer(bframebuf[key] ?? null)

				// Create new textures with updated canvas size
				atexture[key] = create_texture()
				btexture[key] = create_texture()
				aframebuf[key] = create_frame_buffer(atexture[key])
				bframebuf[key] = create_frame_buffer(btexture[key])
				flip[key] = false
			}
		}
	}

	const resize = () => {
		// Update WebGL viewport
		gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)

		// Recreate all textures and framebuffers with new dimensions
		recreate_textures()

		// Reset timing to avoid glitches
		const now = Date.now()
		const elapsed = prev_draw_time - first_draw_time
		first_draw_time = now - elapsed
		prev_draw_time = now

		// Force a redraw
		if (!is_playing) {
			draw()
		}
	}

	const destroy = () => {
		// Stop animation loop
		is_playing = false

		// Delete all programs and shaders
		possible_keys.forEach((key) => {
			if (program[key]) {
				gl.deleteProgram(program[key])
				program[key] = null
			}
			if (vertex_shaders[key]) {
				gl.deleteShader(vertex_shaders[key])
				vertex_shaders[key] = null
			}
			if (fragment_shaders[key]) {
				gl.deleteShader(fragment_shaders[key])
				fragment_shaders[key] = null
			}
		})

		// Delete all textures and framebuffers
		for (const key of ['A', 'B', 'C', 'D'] as const) {
			if (atexture[key]) {
				gl.deleteTexture(atexture[key])
				atexture[key] = null
			}
			if (btexture[key]) {
				gl.deleteTexture(btexture[key])
				btexture[key] = null
			}
			if (aframebuf[key]) {
				gl.deleteFramebuffer(aframebuf[key])
				aframebuf[key] = null
			}
			if (bframebuf[key]) {
				gl.deleteFramebuffer(bframebuf[key])
				bframebuf[key] = null
			}
		}

		// Delete quad buffer
		if (quad_buffer) {
			gl.deleteBuffer(quad_buffer)
			quad_buffer = null
		}

		// Clear event listeners (store references during setup to remove them)
		// Note: These won't be removed since we don't have references to the handlers
		// If you need to remove them, store handler references in setup()

		// Lose WebGL context to free GPU memory
		const lose_context = gl.getExtension('WEBGL_lose_context')
		if (lose_context) {
			lose_context.loseContext()
		}

		// Clear callback
		on_draw_callback = undefined
	}

	setup()

	return {
		data: {
			set_common,
			set_buffer_a,
			set_buffer_b,
			set_buffer_c,
			set_buffer_d,
			set_image,
			set_on_draw,
			set_stream,
			set_volume,
			add_texture,
			time,
			reset,
			pause,
			play,
			resize,
			destroy,
			is_playing() {
				return is_playing
			},
		},
		error: null,
	}
}
