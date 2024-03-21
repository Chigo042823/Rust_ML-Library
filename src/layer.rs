use std::vec;

use rand::{thread_rng, Rng};

use crate::{activation::{Activation, ActivationFunction}, convolution_params::{ConvParams, PaddingType}, dense_params::{self, DenseParams}};
use serde_derive::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayerType {
    Dense,
    Convolutional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub activation: Activation,
    pub layer_type: LayerType,
    pub conv_params: Option<ConvParams>,
    pub dense_params: Option<DenseParams>,
}

impl Layer {
    pub fn dense(nodes: [usize; 2], activation_fn: ActivationFunction) -> Self {
        let dense_params = Some(DenseParams::new(nodes[0], nodes[1]));

        let layer = Layer {
            dense_params,
            activation: Activation::new(activation_fn),
            layer_type: LayerType::Dense,
            conv_params: None
        };
        layer
    }

    pub fn conv(kernel: usize, padding_type: PaddingType, stride: usize, activation_fn: ActivationFunction) -> Self {
        let conv_params = Some(ConvParams::new(kernel, padding_type, stride));
        let layer = Layer {
            dense_params: None,
            activation: Activation::new(activation_fn),
            layer_type: LayerType::Convolutional,
            conv_params
        };
        layer
    }

    pub fn conv_forward(&mut self, inputs: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
        let params = self.conv_params.as_mut().unwrap();
        params.inputs = inputs.clone();
        if params.padding_type == PaddingType::Valid {
            params.data = params.inputs.clone();
        }
        params.add_padding();
        let output_dims = params.get_output_dims();
        let mut weighted_inputs = vec![vec![0.0; output_dims[0]]; output_dims[1]];
        let img = params.data.clone();

        for j in (0..weighted_inputs.len()) { //each img row
            if j + params.kernel > params.data.len() {
                break;
            }
            for k in (0..weighted_inputs[j].len()) { //each img column
                if k + params.kernel > params.data[0].len() {
                    break;
                }
                for kern_row in 0..params.kernel { //Kernel rows
                    for kern_col in 0..params.kernel { //Kernel Columns
                        weighted_inputs[j][k] += (img[j * params.stride + kern_row][k * params.stride + kern_col] * params.weights[kern_row][kern_col]);
                        weighted_inputs[j][k] += params.bias;
                    }
                }
            }
        }

        let mut activation = vec![vec![0.0; output_dims[0]]; output_dims[1]];
        for j in 0..weighted_inputs.len() { 
            for k in 0..weighted_inputs[j].len() { 
                activation[j][k] = self.activation.function(weighted_inputs[j][k]);
            }
        }
        params.outputs = activation.clone();
        activation
    }

    pub fn dense_forward(&mut self, inputs: Vec<f64>) -> Vec<f64> {
        self.dense_params.as_mut().unwrap().inputs = inputs.clone();
        let mut weighted_inputs = self.dense_params.as_mut().unwrap().biases.clone();
        for i in 0..self.dense_params.as_mut().unwrap().nodes_out {
            for j in 0..self.dense_params.as_mut().unwrap().nodes_in {
                weighted_inputs[i] += inputs[j] * self.dense_params.as_mut().unwrap().weights[j][i];
            }
        }

        let mut activation = vec![0.0; self.dense_params.as_mut().unwrap().nodes_out];
        for i in 0..self.dense_params.as_mut().unwrap().nodes_out {
            activation[i] = self.activation.function(weighted_inputs[i]);
        }
        self.dense_params.as_mut().unwrap().outputs = activation.clone();
        activation
    }

    pub fn conv_backward(&mut self, errors: Vec<Vec<f64>>, learning_rate: f64) -> Vec<Vec<f64>> {
        let params = self.conv_params.as_mut().unwrap();
        let mut delta_output = errors.clone();
        for i in 0..delta_output.len() {
            for j in 0..delta_output[i].len() {
                delta_output[i][j] *= self.activation.derivative(params.outputs[i][j].clone());
            }
        }

        let mut weight_gradients = vec![vec![0.0; params.kernel]; params.kernel];
        let img = params.data.clone();

        for j in (0..delta_output.len()) { //each img row
            if j + params.kernel > params.data.len() {
                break;
            }
            for k in (0..delta_output[j].len()) { //each img column
                if k + params.kernel > params.data[0].len() {
                    break;
                }
                for kern_row in 0..params.kernel { //Kernel rows
                    for kern_col in 0..params.kernel { //Kernel Columns
                        weight_gradients[kern_row][kern_col] += (img[j * params.stride + kern_row][k * params.stride + kern_col] * delta_output[j][k]);
                    }
                }
            }
        }

        for i in 0..params.weights.len() {
            for j in 0..params.weights[i].len() {
                for k in 0..delta_output.len() {
                    for l in 0..delta_output[k].len() {
                        params.weights[i][j] -= learning_rate *  delta_output[k][l];
                    }
                }
            }
        }

        // Update bias using the average of the gradients
        let mut avg_bias_gradient = 0.0;
        for i in 0..delta_output.len() {
            for j in 0..delta_output[i].len() {
                avg_bias_gradient += delta_output[i][j];
            }
        }
        avg_bias_gradient /= (delta_output.len() * delta_output[0].len()) as f64;
        params.bias -= learning_rate * avg_bias_gradient;

        let mut next_delta = vec![]; //3x3
        //full convolution with kernel rotated 180 degrees
        let padded_gradients = Self::add_padding_matrix(params.kernel - 1, &delta_output);
        //padded_gradients = 4x4
        for j in (0..padded_gradients.len()) { //each img row
            if j + params.kernel > padded_gradients.len() {
                break;
            }
            let mut gradient_row = vec![];
            for k in (0..padded_gradients[j].len()) { //each img column
                if k + params.kernel > padded_gradients[0].len() {
                    break;
                }
                let mut sum = 0.0;
                for kern_row in 0..params.kernel { //Kernel rows
                    for kern_col in 0..params.kernel { //Kernel Columns
                        sum += (params.weights[kern_row][kern_col] * padded_gradients[j * params.stride + kern_row][k * params.stride + kern_col]);
                    }
                }
                gradient_row.push(sum);
            }
            next_delta.push(gradient_row);
        }
        next_delta
    }

    pub fn add_padding_matrix(padding: usize, matrix: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
        let height = matrix.len();
        let width = matrix[0].len();
        let padded_height = height + 2 * padding;
        let padded_width = width + 2 * padding;
        
        let mut padded_image = vec![vec![0.0; padded_width]; padded_height];

        for i in 0..height {
            for j in 0..width {
                padded_image[i + padding][j + padding] = matrix[i][j];
            }
        }
        padded_image
    }

    pub fn dense_backward(&mut self, errors: Vec<f64>, learning_rate: f64) -> Vec<f64> {
        let mut delta_output = errors.clone();
        for i in 0..delta_output.len() {
            delta_output[i] *= self.activation.derivative(self.dense_params.as_mut().unwrap().outputs[i].clone());
            // delta_output[i] = delta_output[i].min(5.0);
        }

        for i in 0..self.dense_params.as_mut().unwrap().weights.len() {
            for j in 0..self.dense_params.as_mut().unwrap().weights[i].len() {
                self.dense_params.as_mut().unwrap().weights[i][j] -= learning_rate * (self.dense_params.as_mut().unwrap().inputs[i] * delta_output[j]);
            }
        }

        for i in 0..self.dense_params.as_mut().unwrap().biases.len() {
            self.dense_params.as_mut().unwrap().biases[i] -= learning_rate * delta_output[i];
        }

        let mut next_delta = vec![0.0; self.dense_params.as_mut().unwrap().nodes_in];
        for i in 0..self.dense_params.as_mut().unwrap().weights.len() {
            for j in 0..self.dense_params.as_mut().unwrap().weights[i].len() {
                next_delta[i] += (self.dense_params.as_mut().unwrap().weights[i][j] * delta_output[j] * self.dense_params.as_mut().unwrap().inputs[i]);
            }   
        }

        next_delta
    }

    pub fn get_weights(&self) -> Vec<Vec<f64>> {
        self.dense_params.as_ref().unwrap().weights.clone()
    }

    pub fn get_biases(&self) -> Vec<f64> {
        self.dense_params.as_ref().unwrap().biases.clone()
    }

    pub fn get_nodes(&self) -> usize {
        self.dense_params.as_ref().unwrap().nodes_out.clone()
    }

    pub fn get_input_nodes(&self) -> usize {
        self.dense_params.as_ref().unwrap().nodes_in.clone()
    }

    pub fn get_outputs(&self) -> Vec<f64> {
        self.dense_params.as_ref().unwrap().outputs.clone()
    }

    pub fn get_layer_type(&self) -> LayerType {
        self.layer_type.clone()
    }

    pub fn reset(&mut self) {
        self.dense_params.as_mut().unwrap().inputs = vec![];
        self.dense_params.as_mut().unwrap().outputs = vec![];
    }

    pub fn set_params(&mut self, weights: Vec<Vec<f64>>, biases: Vec<f64>) {
        self.dense_params.as_mut().unwrap().weights = weights;
        self.dense_params.as_mut().unwrap().biases = biases;
    }
}