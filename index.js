import * as THREE from 'three';
import {OBJLoader } from 'three/addons/loaders/OBJLoader.js';
import { MTLLoader } from 'three/addons/loaders/MTLLoader.js';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { GUI } from 'three/addons/libs/lil-gui.module.min.js'
const renderer = new THREE.WebGLRenderer();
renderer.shadowMap.enabled = true; //enable shadows
renderer.setSize(window.innerWidth, window.innerHeight);

document.body.appendChild(renderer.domElement);

const scene = new THREE.Scene();

const camera = new THREE.PerspectiveCamera(
    75,
    window.innerWidth / window.innerHeight,
    .1,
    1000
)

//lighting below
const ambientLight = new THREE.AmbientLight(0x333333);

const spotLight = new THREE.SpotLight(0xFFFFFF);
spotLight.position.set(50,50,50);
spotLight.castShadow = true;
spotLight.angle = .2;
spotLight.intensity = 800;
scene.add(spotLight,ambientLight);

const sLightHelper = new THREE.SpotLightHelper(spotLight);
// scene.add(sLightHelper);




//intersection plane

const plane = new THREE.PlaneGeometry(64,64);
const planeMaterial = new THREE.MeshStandardMaterial({color:0x66EBBB});
const planeObj = new THREE.Mesh(plane,planeMaterial);
planeObj.receiveShadow = true;
planeObj.rotateX(-Math.PI/2);
planeObj.name = "interactable"
scene.add(planeObj);

const selectedMesh = new THREE.PlaneGeometry(8,8);
const selectedMeshMaterial = new THREE.MeshStandardMaterial({color: 0xFFEA00, visible: false})
const selectedObj = new THREE.Mesh(selectedMesh, selectedMeshMaterial);
selectedObj.rotateX(-Math.PI / 2);
scene.add(selectedObj);


const grid = new THREE.GridHelper(64);
grid.translateY(1)
scene.add(grid);

const raycaster = new THREE.Raycaster();
const pointer = new THREE.Vector2();

window.addEventListener("click", function(e) {
    
    pointer.x = ( e.clientX / window.innerWidth ) * 2 - 1;
	pointer.y = - ( e.clientY / window.innerHeight ) * 2 + 1;
    raycaster.setFromCamera(pointer, camera);
    const intersection = raycaster.intersectObject(planeObj)[0]
    if(intersection.object.name == 'interactable')
    {
        console.log(pointer)
        console.log(intersection.point)
    }
    
})
// const helper = new THREE.CameraHelper( spotLight.shadow.camera );
// scene.add( helper );

//orbit controls
const orbit = new OrbitControls(camera, renderer.domElement)

camera.position.set(0, 40, 40);
orbit.update()

const axesHelper = new THREE.AxesHelper(100);
scene.add(axesHelper);
const gui = new GUI();

const options = {
    sphereColor: '#ffea00',
    wireframe: false,
    speed: 0.01,
    angle: 0.4,
    penumbra: 1,
    intensity: 800
};

gui.addColor(options, 'sphereColor').onChange(function(e){
    sphere.material.color.set(e);
});

gui.add(options, 'wireframe').onChange(function(e){
    sphere.material.wireframe = e;
});

gui.add(options, 'speed', 0, 1000);

gui.add(options, 'angle', 0, 1);
gui.add(options, 'penumbra', 0, 1);
gui.add(options, 'intensity', 0, 1000
);


//load materials and objects
const mtlloader = new MTLLoader();

mtlloader.load('static/Pawn.mtl',
                function(mtl) {
                    mtl.preload()
                        const loader = new OBJLoader();
                        loader.setMaterials(mtl);
                        loader.load('static/Pawn.obj',
                            function ( object ) {
                                object.scale.set(.5,.5,.5);
                                object.castShadow = true;
                                object.traverse(function(child) {
                                    //traverse through children and set them to castshadows as well
                                    child.castShadow = true;
                                })
                                scene.add( object );

                            },
                            // called when loading is in progress
                            function ( xhr ) {

                                console.log( ( xhr.loaded / xhr.total * 100 ) + '% loaded' );

                            },
                            // called when loading has errors
                            function ( error ) {

                                console.log( 'An error happened', error);

                            }
                        );
                }
)


function animate() {
    spotLight.angle = options.angle;
    spotLight.penumbra = options.penumbra;
    spotLight.intensity = options.intensity;
    sLightHelper.update();

	renderer.render( scene, camera );
}
console.log(scene)

renderer.setAnimationLoop( animate );

