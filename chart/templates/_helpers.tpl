{{/*
Expand the name of the chart.
*/}}
{{- define "kora.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "kora.fullname" -}}
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
Standard labels.
*/}}
{{- define "kora.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{ include "kora.selectorLabels" . }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels.
*/}}
{{- define "kora.selectorLabels" -}}
app.kubernetes.io/name: {{ include "kora.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Return the proper image name.
*/}}
{{- define "kora.image" -}}
{{- $tag := .Values.image.tag | default (printf "v%s" .Chart.AppVersion) -}}
{{- printf "%s/%s:%s" .Values.image.registry .Values.image.repository $tag -}}
{{- end }}

{{/*
Build DATABASE_URL from components.
Priority: url > host/port/user/password/name.
*/}}
{{- define "kora.databaseUrl" -}}
{{- if .Values.database.url }}
{{- .Values.database.url }}
{{- else }}
{{- printf "postgres://%s:%s@%s:%v/%s" .Values.database.user .Values.database.password .Values.database.host (int .Values.database.port) .Values.database.name }}
{{- end }}
{{- end }}

{{/*
Return the name of the database secret.
*/}}
{{- define "kora.databaseSecretName" -}}
{{- if .Values.database.existingSecret }}
{{- .Values.database.existingSecret }}
{{- else }}
{{- include "kora.fullname" . }}-db
{{- end }}
{{- end }}

{{/*
Validate values — fail early on misconfiguration.
*/}}
{{- define "kora.validateValues" -}}
{{- if and (not .Values.database.host) (not .Values.database.url) (not .Values.database.existingSecret) }}
{{- fail "database: you must set one of database.host, database.url, or database.existingSecret" }}
{{- end }}
{{- if and .Values.database.host (not .Values.database.url) (not .Values.database.existingSecret) (not .Values.database.password) }}
{{- fail "database: database.password is required when using database.host (use database.existingSecret for production)" }}
{{- end }}
{{- end }}
