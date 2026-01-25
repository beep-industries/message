{{/*
Expand the name of the chart.
*/}}
{{- define "message.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "message.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "message.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "message.labels" -}}
helm.sh/chart: {{ include "message.chart" . }}
{{ include "message.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "message.selectorLabels" -}}
app.kubernetes.io/name: {{ include "message.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "message.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "message.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Database secret name
*/}}
{{- define "message.databaseSecretName" -}}
{{- if .Values.database.existingSecret }}
{{- .Values.database.existingSecret }}
{{- else }}
{{- include "message.fullname" . }}-db
{{- end }}
{{- end }}

{{/*
JWT secret name
*/}}
{{- define "message.jwtSecretName" -}}
{{- if .Values.jwt.existingSecret }}
{{- .Values.jwt.existingSecret }}
{{- else }}
{{- include "message.fullname" . }}-jwt
{{- end }}
{{- end }}

{{/*
RabbitMQ secret name
*/}}
{{- define "message.rabbitmqSecretName" -}}
{{- if .Values.rabbitmq.existingSecret }}
{{- .Values.rabbitmq.existingSecret }}
{{- else }}
{{- include "message.fullname" . }}-rabbitmq
{{- end }}
{{- end }}
